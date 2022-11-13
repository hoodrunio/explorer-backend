use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::others::{DenomAmount, Pagination, PaginationConfig};
use crate::chain::Chain;

impl Chain {
    /// Makes a request to the RPC node.
    pub(super) async fn rpc_request<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&'static str, String)],
    ) -> Result<T, String> {
        // Create the URL request to.
        let url = format!("{}{}", self.rpc_url, path);

        match self.client.get(url).query(query).send().await {
            Ok(res) => match res.json::<RPCResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RPCResponse::Success(res) => Ok(res.result),
                    RPCResponse::Error(res) => Err(res.error.data),
                },
                Err(_) => Err("Cannot parse JSON.".to_string()),
            },
            Err(_) => Err("Unsuccessful request.".to_string()),
        }
    }

    /// Makes a request to the REST API node.
    pub(super) async fn rest_api_request<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&'static str, String)],
    ) -> Result<T, String> {
        // Create the URL request to.
        let url = format!("{}{}", self.rest_url, path);

        match self.client.get(url).query(query).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    match res.json::<RestResponse<T>>().await {
                        Ok(res_json) => match res_json {
                            RestResponse::Success(res_json) => Ok(res_json),
                            RestResponse::Error { message, details: _ } => Err(message),
                        },
                        Err(error) => {
                            println!("{:#?}", error);
                            Err("Cannot parse JSON error response.".to_string())
                        }
                    }
                } else {
                    match res.json::<RestResponse<T>>().await {
                        Ok(res_json) => match res_json {
                            RestResponse::Success(res_json) => Ok(res_json),
                            RestResponse::Error { message, details: _ } => Err(message),
                        },
                        Err(error) => {
                            println!("{:#?}", error);
                            Err("Cannot parse JSON error response.".to_string())
                        }
                    }
                }
            }
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
    pub jsonrpc: String,
    pub id: isize,
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
