use gloo_net::Error as GlooNetError;
use hex::FromHexError;
use json::Error as JsonError;
use serde_json as json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    /// Could not create RPC: possibly invalid URL
    #[error("could not create RPC: possibly invalid URL")]
    InvalidRPC,
    #[error(transparent)]
    GlooNetError(#[from] GlooNetError),
    /// Error while parsing json
    #[error(transparent)]
    JsonError(#[from] JsonError),
    /// Error while parsing json: invalid hex value
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    /// Error while parsing json: invalid account
    #[error("error while parsing json: invalid account")]
    InvalidAccount,
    /// Error while parsing json: invalid integer
    #[error("error while parsing json: invalid integer")]
    InvalidInteger,
    /// error while parsing json: unexpected data type
    #[error("error while parsing json: unexpected data type")]
    InvalidJsonDataType,
    /// The returned data is invalid
    #[error("the returned data is invalid")]
    InvalidData,
    /// Cannot publish block of type `legacy`
    #[error("cannot publish block of type 'legacy'")]
    LegacyBlockType,
}
