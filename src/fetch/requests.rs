use reqwest::{Client, Method};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{chain::Chain, logging::{log_api, LogLevel}};

impl Chain {
    /// Makes a request to the RPC node.
    pub(super) async fn rpc_request<T: DeserializeOwned>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
        // Create the URL request to.
        let url = format!("{}{}", self.config.rpc_url, path);
        log_api(LogLevel::DEBUG, &format!("Making RPC request to: {}", url));

        match self.client.get(&url).query(query).send().await {
            Ok(res) => match res.json::<RPCResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RPCResponse::Success(res) => {
                        log_api(LogLevel::INFO, &format!("Successful RPC request to: {}", url));
                        Ok(res.result)
                    },
                    RPCResponse::Error(res) => {
                        log_api(LogLevel::ERROR, &format!("RPC error response from {}: {}", url, res.error.data));
                        Err(res.error.data)
                    },
                },
                Err(error) => {
                    log_api(LogLevel::ERROR, &format!("Failed to parse JSON from {}: {}", url, error));
                    Err(format!("Cannot parse JSON.\nURL requested: {url}\nError Message:\n{error}"))
                },
            },
            Err(e) => {
                log_api(LogLevel::ERROR, &format!("Failed to make request to {}: {}", url, e));
                Err(format!("Cannot make a request to `{url}`."))
            },
        }
    }

    /// Makes a request to the REST API node.
    pub(super) async fn rest_api_request<T: DeserializeOwned>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
        // Create the URL request to.
        let url = format!("{}{}", self.config.rest_url, path);
        log_api(LogLevel::DEBUG, &format!("Making REST API request to: {}", url));

        match self.client.get(&url).query(query).send().await {
            Ok(res) => match res.json::<RestResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RestResponse::Success(res_json) => {
                        log_api(LogLevel::INFO, &format!("Successful REST API request to: {}", url));
                        Ok(res_json)
                    },
                    RestResponse::Error { message, details } => {
                        let error_msg = if !details.is_empty() {
                            format!("{}: {}", message, details.join(", "))
                        } else {
                            message.clone()
                        };
                        log_api(LogLevel::ERROR, &format!("REST API error response from {}: {}", url, error_msg));
                        Err(message)
                    },
                },
                Err(error) => {
                    log_api(LogLevel::ERROR, &format!("Failed to parse JSON from {}: {}", url, error));
                    Err(format!("Cannot parse JSON.\nURL requested: {url}\nError Message:\n{error}"))
                },
            },
            Err(e) => {
                log_api(LogLevel::ERROR, &format!("Failed to make request to {}: {}", url, e));
                Err(format!("Cannot make a request to `{url}`."))
            },
        }
    }

    /// Makes a request to the ARCHIVE REST API node.
    pub(super) async fn archive_api_request<T: DeserializeOwned>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
        // Create the URL request to.
        let url = format!("{}{}", self.config.archive_url, path);
        log_api(LogLevel::DEBUG, &format!("Making Archive API request to: {}", url));

        match self.client.get(&url).query(query).send().await {
            Ok(res) => match res.json::<RestResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RestResponse::Success(res_json) => {
                        log_api(LogLevel::INFO, &format!("Successful Archive API request to: {}", url));
                        Ok(res_json)
                    },
                    RestResponse::Error { message, details } => {
                        let error_msg = if !details.is_empty() {
                            format!("{}: {}", message, details.join(", "))
                        } else {
                            message.clone()
                        };
                        log_api(LogLevel::ERROR, &format!("Archive API error response from {}: {}", url, error_msg));
                        Err(message)
                    },
                },
                Err(error) => {
                    log_api(LogLevel::ERROR, &format!("Failed to parse JSON from {}: {}", url, error));
                    Err(format!("Cannot parse JSON.\nURL requested: {url}\nError Message:\n{error}"))
                },
            },
            Err(e) => {
                log_api(LogLevel::ERROR, &format!("Failed to make request to {}: {}", url, e));
                Err(format!("Cannot make a request to `{url}`."))
            },
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
        
        log_api(LogLevel::DEBUG, &format!("Making JSON-RPC request to: {}", url));

        match self.client.post(&url).body(body.clone()).send().await {
            Ok(res) => match res.json::<JsonRpcResponse<T>>().await {
                Ok(res_json) => match res_json {
                    JsonRpcResponse::Success(res) => {
                        log_api(LogLevel::INFO, &format!("Successful JSON-RPC request to: {}", url));
                        Ok(res.result)
                    },
                    JsonRpcResponse::Error(res) => {
                        log_api(LogLevel::ERROR, &format!("JSON-RPC error response from {}: {}", url, res.error.message));
                        Err(res.error.message)
                    },
                },
                Err(error) => {
                    log_api(LogLevel::ERROR, &format!("Failed to parse JSON from {}: {}", url, error));
                    Err(format!("Cannot parse JSON.\nURL requested: {url}\nError Message:\n{error}"))
                },
            },
            Err(e) => {
                log_api(LogLevel::ERROR, &format!("Failed to make request to {}: {}", url, e));
                Err(format!("Cannot make a request to `{url}`."))
            },
        }
    }

    // Makes a request to the External Resource
    pub(super) async fn external_rest_api_req<T: DeserializeOwned>(
        &self,
        url: String,
        query: &[(&'static str, String)],
        method: Method,
    ) -> Result<T, String> {
        log_api(LogLevel::DEBUG, &format!("Making external request to: {}", url));

        let request = self.client.request(method, &url).query(query);

        match request.send().await {
            Ok(res) => match res.json::<T>().await {
                Ok(res_json) => {
                    log_api(LogLevel::INFO, &format!("Successful external request to: {}", url));
                    Ok(res_json)
                },
                Err(error) => {
                    log_api(LogLevel::ERROR, &format!("Failed to parse JSON from {}: {}", url, error));
                    Err(format!("Cannot parse JSON.\nURL requested: {url}\nError Message:\n{error}"))
                },
            },
            Err(e) => {
                log_api(LogLevel::ERROR, &format!("Failed to make request to {}: {}", url, e));
                Err(format!("Cannot make a request to `{url}`."))
            },
        }
    }

    pub(super) async fn coingecko_rest_client<T: DeserializeOwned>(&self, url: String, query: &[(&'static str, String)]) -> Result<T, String> {
        let full_path = format!("https://api.coingecko.com/api/v3{url}");
        self.external_rest_api_req(full_path, query, Method::GET).await
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum RestResponse<T> {
    Success(T),
    Error {
        message: String,
        details: Vec<String>,
    },
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
    pub code: i32,
    pub message: String,
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
