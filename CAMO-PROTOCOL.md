# Camo Nano Protocol

Note that this is *not* meant to serve as a formal specification, and is only informal documentation.

## Definitions

Let:

1. $x \mathbin\Vert y$ denote concatenation;
2. $G$ denote the ed25519 generator point;
3. $(a, A)$ denote an ed25519 keypair where $A = a \cdot G$;
4. $H_{32}(x)$, or simply $H(x)$, denote the blake2b hashing algorithm outputting a 32-byte digest;
5. $H_{64}(x)$ denote the blake2b hashing algorithm outputting a 64-byte digest;
6. $H_{checksum}(x)$ denote the blake2b hashing algorithm outputting a 5-byte digest;
7. $H_{category}(x, i)$ be defined as $H(i \mathbin\Vert x)$;
8. $H_{seed}(x, i)$ be defined as $H(x \mathbin\Vert i)$;
9. $H_{s}(x)$ be defined as $H_{64}(x)$, where the first 32 bytes of the output are interpreted as an ed25519 scalar following standard safety guidelines, such as bit clamping;
10. $H_{si}(x, i)$ be defined as $H_{s}(H_{seed}(x, i))$;
11. ${EncodeBase32}(x)$ denote Nano's encode-to-base32 algorithm;
12. ${DecodeBase32}(x)$ denote Nano's decode-from-base32 algorithm;

. . . where $i$ denotes a 32-bit unsigned integer encoded as big-endian bytes.

Note that only $H_{category}(x, i)$ is unique to this protocol; The rest are already in use by the standard Nano protocol (albeit some under different names), and can be re-used.

## Keypairs

Let:
 * $i$ be a 32-bit unsigned integer, denoting this camo account's index;
 * $s_{master}$ denote the wallet's seed, encoded as 32 bytes;

First, we define some wallet-specific constants. Let:
 * $s_{spend} = H_{category}(s_{master}, 0)$;
 * $s_{view} = H_{category}(s_{master}, 1)$;
 * $k_{master} = H_{si}(s_{spend}, 0)$;
 * $K_{master} = k_{master} \cdot G$;

Next, we calculate the account-specific seed. Let:
 * $s = H_{64}(H_{seed}(s_{view}, i))$;

Next, we calculate our private keys, $k_{spend}$ and $k_{view}$. Let:
 * $k_{spend} = k_{master} + H_{s}(s_{[0:32]})$;
 * $k_{view} = H_{s}(s_{[32:64]})$;

Finally, we calculate our public keys, $K_{spend}$ and $K_{view}$. Let:
 * $K_{spend} = k_{spend} \cdot G = K_{master} + (H_{s}(s_{[0:32]}) \cdot G)$;
 * $K_{view} = k_{view} \cdot G$;

### View Keys

Note that, given only $s_{view}$ and $K_{master}$, a "view-only" client can be created for this wallet. Using these, $k_{view}$, $K_{spend}$, and $K_{view}$ can be determined for all camo accounts in the wallet. Let:
 * $s = H_{64}(H_{seed}(s_{view}, i))$.
 * $k_{view} = H_{s}(s_{[32:64]})$.
 * $K_{spend} = K_{master} + (H_{s}(s_{[0:32]}) \cdot G)$.
 * $K_{view} = k_{view} \cdot G$;

A view-only client would be able to view the transaction history of the wallet, but would *not* be able to spend from it.

The mechanism by which to encode and distribute view keys is not yet defined.

## Protocol Versions

The protocol is designed to be flexible, and upgradable without breaking older software. Up to 8 distinct versions can be supported, with currently only version 1 being implemented. Note that a higher version is not necessarily "better", just "newer".

The versions which any particular camo account supports is signaled by toggling bits in 1 byte, $v$, at the front of the address. Examples:
* If the account only supports version 1, then $v$ = `0b_00000001` = `0x01`.
* If the account only supports version 4, then $v$ = `0b_00001000` = `0x08`.
* If the account supports versions 1, 2, and 4, then $v$ = `0b_00001011` = `0x0b`.
* If the account supports versions 2, 5, and 7, then $v$ = `0b_01010010` = `0x52`.

The "preferred" version is defined as the highest version being signaled. For example, the preferred version of $v$ = `0b_01010010` = `0x52` would be 7. If, in this example, the sender's software only supports up to version 6, then the sender would select the next highest version, in this case version 5.

If the sender's software does not support any of the signaled versions on a camo account, then no transaction can be made. In that case, an appropriate error should be returned to the user.

## Accounts

A Camo Nano address, $C$, is defined as the following. Let:
 * $C_{data} = v \mathbin\Vert K_{spend} \mathbin\Vert K_{view}$;
 * $C_{checksum} = H_{checksum}(C_{data})$;
 * $C = "camo\_" \mathbin\Vert {EncodeBase32}(C_{data} \mathbin\Vert C_{checksum})$;

To decode a Camo Nano address, $C$, reverse the above process:
 * Check $C_{[0:5]} \stackrel{?}{=} "camo\_"$;
 * Let $C_{raw} = {DecodeBase32}(C_{[5:]})$;
 * Parse $C_{raw}$ to obtain $C_{data}$ and $C_{checksum}$;
 * Check $C_{checksum} \stackrel{?}{=} H_{checksum}(C_{data})$;
 * Parse $C_{data}$ to obtain $v$, $K_{spend}$, and $K_{view}$;

An example Camo Nano address is `camo_168be68tsxk1o8xferck89gj75kzk8fpbhote77ed1db975htuf11psgpwq9wabcxdjssycim6tidgkau48x6tgcqnsnxj341mamjpoy8umaz45c`, created with seed `c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8`, at index `5`, and $v$ = `0x01`.

## Transactions

### Sending

If a user, using a Nano account $A_{1}$ with private key $a$ and most recent ("frontier") block $B$, wishes to send $n$ coins to camo account $(v, K_{spend}, K_{view})$, then let:
 * $r = H_{s}(a \mathbin\Vert H(B) \mathbin\Vert K_{spend}$);
 * $R = r \cdot G$;
 * $Q = r \cdot K_{view}$;
 * $k_{shared} = H_{si}(Q, 0)$;
 * $K_{masked} = K_{spend} + (k_{shared} \cdot G)$;

Using Nano account $A_{2}$, preferably such that $A_{1} \ne A_{2}$, with the "representative" field set to the Nano account with public key $R$, send a payment of low value $(n_{notify} < n)$ to the Nano account with public key $K_{spend}$. This will be referred to as a "notification" transaction/payment.

Using $A_{1}$, send $(n - n_{notify})$ coins to $K_{masked}$. This will be referred to as a "camo" transaction/payment.

### Receiving

Given a keypair as defined in the [Keypairs](#keypairs) section, check for incoming notification payments to the Nano account with public key $K_{spend}$. Receive the notification payments.

For each notification, extract $R$, and let:
 * $Q = k_{view} \cdot R$;
 * $k_{shared} = H_{si}(Q, 0)$;
 * $k_{masked} = k_{spend} + k_{shared}$;
 * $K_{masked} = k_{masked} \cdot G$;

Check the Nano account with keypair $(k_{masked}, K_{masked})$ for incoming camo payments, and receive them.

### Notes
$r$ does not necessarily have to be calculated in this way. All that matters is that it is secret, and unique to every camo payment. However, using a standard pseudo-random algorithm is useful if $r$ ever needs to be recovered by the sender.

The calculated $Q$ value is the same between the sender and recipient, since $Q = K_{view} \cdot r = k_{view} \cdot R = k_{view} \cdot r \cdot G$. Since calculating $K_{masked}$ requires knowledge of $Q$, which itself requires knowledge of either $r$ or $k_{view}$, no outside observer can calculate $K_{masked}$.

The sender can send a notification to the recipient at any point during or after the camo payment, and even send duplicate notifications, as long as the sender knows $R$.

This system is capable of sending to camo accounts through wallet software which does not support them, though it's not necessarily recommended. Use external software to calculate $K_{masked}$, and any other values as needed. Then:
 * To send a camo payment, create a transaction sending coins to $K_{masked}$.
 * To send a notification, create a transaction sending a small number of coins to $K_{spend}$, with the "representative" field set to $R$.

 When dealing with notifications, care must be taken to ensure that all coins are accounted for. Well-designed wallet software should consider the following:
 * Camo payments may take longer to confirm than notifications, so it may temporarily appear that a notification has no associated camo payment. Handle "unlinked" notifications carefully, and do not immediately ignore them.
 * A "rescan" feature should be provided to allow users to rescan the notifications they've received, so that mishandled payments, and coins in restore-from-seed wallets, can be recovered.
 * "Notifier" and "sender" accounts should be chosen wisely. The easy solution is to use one account for both, but that harms privacy. Users should be able to make tradeoffs between privacy, ease-of-use, and user-control, through settings with sane defaults.