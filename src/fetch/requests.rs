use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::chain::Chain;

pub enum RequestType {
    Rpc,
    Rest,
    JsonRpc,
}

impl Chain {
    // pub(super) async fn request<T: DeserializeOwned>(&self, r_type: RequestType, path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
    //     let res = match r_type {
    //         RequestType::Rpc => {
    //             let url = format!("{}{}", self.config.rpc_url, path);
    //             self.client.get(&url).query(query).send().await
    //         },
    //         RequestType::Rest => {
    //             let url = format!("{}{}", self.config.rest_url, path);
    //             self.client.get(&url).query(query).send().await
    //         },
    //         RequestType::JsonRpc => {
    //             let url = format!("{}{}", self.config.json_rpc_url, path);
    //             self.client.get(&url).body(body).send().await
    //         },
    //     };
    //
    //     match res {
    //         Ok
    //     }
    // }
    /// Makes a request to the RPC node.
    pub(super) async fn rpc_request<T: DeserializeOwned>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
        // Create the URL request to.
        let url = format!("{}{}", self.config.rpc_url, path);

        match self.client.get(&url).query(query).send().await {
            Ok(res) => match res.json::<RPCResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RPCResponse::Success(res) => Ok(res.result),
                    RPCResponse::Error(res) => Err(res.error.data),
                },
                Err(error) => Err(format!("Cannot parse JSON.\nURL requested: {url}\nError Message:\n{error}")),
            },
            Err(_) => Err(format!("Cannot make a request to `{url}`.")),
        }
    }

    /// Makes a request to the REST API node.
    pub(super) async fn rest_api_request<T: DeserializeOwned>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
        // Create the URL request to.
        let url = format!("{}{}", self.config.rest_url, path);

        match self.client.get(&url).query(query).send().await {
            Ok(res) => match res.json::<RestResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RestResponse::Success(res_json) => Ok(res_json),
                    RestResponse::Error { message, details: _ } => Err(message),
                },
                Err(error) => Err(format!("Cannot parse JSON.\nURL requested: {url}\nError Message:\n{error}")),
            },
            Err(_) => Err(format!("Cannot make a request to `{url}`.")),
        }
    }

    /// Makes a post request to the JSON RPC node.
    pub(super) async fn jsonrpc_request<T: DeserializeOwned>(&self, body: String) -> Result<T, String> {
        // Create the URL request to.
        let url = self
            .config
            .jsonrpc_url
            .clone()
            .ok_or_else(|| format!("`jsonrpc` key for {} is empty in `Chains.yml` file.", self.config.name))?;

        match self.client.post(&url).body(body).send().await {
            Ok(res) => match res.json::<JsonRpcResponse<T>>().await {
                Ok(res_json) => match res_json {
                    JsonRpcResponse::Success(res) => Ok(res.result),
                    JsonRpcResponse::Error(res) => Err(res.error.message),
                },
                Err(error) => Err(format!("Cannot parse JSON.\nURL requested: {url}\nError Message:\n{error}")),
            },
            Err(_) => Err(format!("Cannot make a request to `{url}`.")),
        }
    }

    // Makes a request to the External Resource
    pub(super) async fn external_rest_api_req<T: DeserializeOwned>(&self, client: &reqwest::Client, method: Method, full_path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
        let request = client.request(method, full_path);

        match request.query(query).send().await {
            Ok(res) => match res.json::<RestResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RestResponse::Success(res_json) => Ok(res_json),
                    RestResponse::Error { message, details: _ } => Err(message),
                },
                Err(error) => Err(format!("Cannot parse JSON.\nURL requested: {full_path}\nError Message:\n{error}")),
            },
            Err(_) => Err(format!("Cannot make a request to `{full_path}`.")),
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
