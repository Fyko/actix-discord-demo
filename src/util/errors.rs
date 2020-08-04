use derive_more::Display;

#[derive(Debug, Display, PartialEq)]
pub enum ApiError {
    CacheError(String),
    CannotDecodeJwtToken(String),
    CannotEncodeJwtToken(String),
}
