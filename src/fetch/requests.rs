use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::chain::Chain;

impl Chain {
    /// Makes a request to the RPC node.
    pub(super) async fn rpc_request<T: DeserializeOwned>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
        // Create the URL request to.
        let url = format!("{}{}", self.inner.rpc_url, path);

        match self.inner.client.get(url).query(query).send().await {
            Ok(res) => match res.json::<RPCResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RPCResponse::Success(res) => Ok(res.result),
                    RPCResponse::Error(res) => Err(res.error.data),
                },
                Err(error) => {
                    eprintln!("{}", error);
                    Err("Cannot parse JSON.".to_string())
                }
            },
            Err(_) => Err("Unsuccessful request.".to_string()),
        }
    }

    /// Makes a request to the REST API node.
    pub(super) async fn rest_api_request<T: DeserializeOwned>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
        // Create the URL request to.
        let url = format!("{}{}", self.inner.rest_url, path);

        match self.inner.client.get(url).query(query).send().await {
            Ok(res) => match res.json::<RestResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RestResponse::Success(res_json) => Ok(res_json),
                    RestResponse::Error { message, details: _ } => Err(message),
                },
                Err(error) => {
                    eprintln!("{:#?}", error);
                    Err(format!("Cannot parse JSON. ({error})"))
                }
            },
            Err(_) => Err("Unsuccessful request.".to_string()),
        }
    }

    /// Makes a post request to the JSON RPC node.
    pub(super) async fn jsonrpc_request<T: DeserializeOwned>(&self, body: String) -> Result<T, String> {
        // Create the URL request to.
        let url = self
            .inner
            .jsonrpc_url
            .ok_or_else(|| format!("`jsonrpc` key for {} is empty in `Chains.yml` file.", self.inner.name))?;

        match self.inner.client.post(url).body(body).send().await {
            Ok(res) => match res.json::<JsonRpcResponse<T>>().await {
                Ok(res_json) => match res_json {
                    JsonRpcResponse::Success(res) => Ok(res.result),
                    JsonRpcResponse::Error(res) => Err(res.error.message),
                },
                Err(error) => {
                    eprintln!("{}", error);
                    Err("Cannot parse JSON.".to_string())
                }
            },
            Err(_) => Err("Unsuccessful request.".to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum RestResponse<T> {
    Success(T),
    Error { message: String, details: Vec<String> },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum RPCResponse<T> {
    Success(RPCSuccessResponse<T>),
    Error(RPCErrorResponse),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RPCSuccessResponse<T> {
    pub result: T,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RPCErrorResponse {
    pub jsonrpc: String,
    pub id: isize,
    pub error: RPCErrorResponseError,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RPCErrorResponseError {
    /// The error code.
    pub code: isize,
    /// The message about error type.
    pub message: String,
    /// Description about error.
    pub data: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum JsonRpcResponse<T> {
    Success(RPCSuccessResponse<T>),
    Error(JsonRpcErrorResponse),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JsonRpcErrorResponse {
    /// The JSON RPC error.
    pub error: JsonRpcErrorResponseError,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JsonRpcErrorResponseError {
    /// The cause of the error.
    pub message: String,
}
