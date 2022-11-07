use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RPCResponse<T> {
    Success(RPCSuccessResponse<T>),
    Error(RPCErrorResponse),
}

#[derive(Deserialize, Debug)]
pub struct RPCSuccessResponse<T> {
    pub jsonrpc: String,
    pub id: isize,
    pub result: T,
}

#[derive(Deserialize, Debug)]
pub struct RPCErrorResponse {
    pub jsonrpc: String,
    pub id: isize,
    pub error: RPCErrorResponseError,
}

#[derive(Deserialize, Debug)]
pub struct RPCErrorResponseError {
    /// The error code.
    pub code: isize,
    /// The message about error type.
    pub message: String,
    /// Description about error.
    pub data: String,
}
