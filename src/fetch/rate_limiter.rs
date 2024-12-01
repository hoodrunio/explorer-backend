use governor::{Quota, RateLimiter};
use nonzero_ext::nonzero;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use serde::de::DeserializeOwned;
use reqwest::Method;

#[derive(Clone)]
pub struct RateLimitedClient {
    client: reqwest::Client,
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    max_retries: u32,
}

impl RateLimitedClient {
    pub fn new(requests_per_second: NonZeroU32, max_retries: u32) -> Self {
        let quota = Quota::per_second(requests_per_second);
        let limiter = Arc::new(RateLimiter::direct(quota));
        
        Self {
            client: reqwest::Client::new(),
            limiter,
            max_retries,
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, String> {
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.max_retries {
            self.limiter.until_ready().await;
            
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        if let Some(retry_after) = response.headers()
                            .get("Retry-After")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                        {
                            sleep(Duration::from_secs(retry_after)).await;
                        } else {
                            sleep(Duration::from_secs(2u64.pow(attempts))).await;
                        }
                    } else {
                        match response.json::<T>().await {
                            Ok(data) => return Ok(data),
                            Err(e) => last_error = Some(format!("JSON parse error: {}", e)),
                        }
                    }
                }
                Err(e) => last_error = Some(format!("Request error: {}", e)),
            }
            
            attempts += 1;
            if attempts < self.max_retries {
                sleep(Duration::from_secs(2u64.pow(attempts))).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| "Max retries exceeded".to_string()))
    }

    pub async fn get_with_query<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)]
    ) -> Result<T, String> {
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.max_retries {
            self.limiter.until_ready().await;
            
            match self.client.get(url).query(query).send().await {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        if let Some(retry_after) = response.headers()
                            .get("Retry-After")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                        {
                            sleep(Duration::from_secs(retry_after)).await;
                        } else {
                            sleep(Duration::from_secs(2u64.pow(attempts))).await;
                        }
                    } else {
                        match response.json::<T>().await {
                            Ok(data) => return Ok(data),
                            Err(e) => last_error = Some(format!("JSON parse error: {}", e)),
                        }
                    }
                }
                Err(e) => last_error = Some(format!("Request error: {}", e)),
            }
            
            attempts += 1;
            if attempts < self.max_retries {
                sleep(Duration::from_secs(2u64.pow(attempts))).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| "Max retries exceeded".to_string()))
    }

    pub async fn post_with_body<T: DeserializeOwned>(
        &self,
        url: &str,
        body: String,
    ) -> Result<T, String> {
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.max_retries {
            self.limiter.until_ready().await;
            
            match self.client.post(url).body(body.clone()).send().await {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        if let Some(retry_after) = response.headers()
                            .get("Retry-After")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                        {
                            sleep(Duration::from_secs(retry_after)).await;
                        } else {
                            sleep(Duration::from_secs(2u64.pow(attempts))).await;
                        }
                    } else {
                        match response.json::<T>().await {
                            Ok(data) => return Ok(data),
                            Err(e) => last_error = Some(format!("JSON parse error: {}", e)),
                        }
                    }
                }
                Err(e) => last_error = Some(format!("Request error: {}", e)),
            }
            
            attempts += 1;
            if attempts < self.max_retries {
                sleep(Duration::from_secs(2u64.pow(attempts))).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| "Max retries exceeded".to_string()))
    }

    pub async fn request_with_query<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
        method: Method,
    ) -> Result<T, String> {
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.max_retries {
            self.limiter.until_ready().await;
            
            match self.client.request(method.clone(), url).query(query).send().await {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        if let Some(retry_after) = response.headers()
                            .get("Retry-After")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                        {
                            sleep(Duration::from_secs(retry_after)).await;
                        } else {
                            sleep(Duration::from_secs(2u64.pow(attempts))).await;
                        }
                    } else {
                        match response.json::<T>().await {
                            Ok(data) => return Ok(data),
                            Err(e) => last_error = Some(format!("JSON parse error: {}", e)),
                        }
                    }
                }
                Err(e) => last_error = Some(format!("Request error: {}", e)),
            }
            
            attempts += 1;
            if attempts < self.max_retries {
                sleep(Duration::from_secs(2u64.pow(attempts))).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| "Max retries exceeded".to_string()))
    }

    // Helper method to get underlying reqwest client
    pub fn inner(&self) -> reqwest::Client {
        self.client.clone()
    }
}