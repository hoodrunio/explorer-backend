use crate::chain::Chain;
use crate::fetch::amount_util::{ChainAmountItemBuilder, TnrDecimal};
use crate::fetch::others::{Pagination, PaginationConfig};
use crate::routes::{TNRAppError, TNRAppErrorType};
use crate::state::State;
use actix_web::web::Data;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

/// The query params that has `page` param.
#[derive(Deserialize)]
pub struct QueryParams {
    /// Page. Indexing starts at 1.\
    /// The value is `1/ONE`, if it is not specified. \
    /// Example values are: 1, 2, 3, 4, etc
    pub page: Option<u8>,
    pub limit: Option<u16>,
}

#[derive(Deserialize)]
pub struct LastCountListsQueryParams {
    pub count: Option<u16>,
}

#[derive(Serialize)]
pub struct OutRestResponse<T> {
    pub value: T,
    /// The count of the pages.
    pub pages: u8,
}

impl<T> OutRestResponse<T> {
    /// Tries to create a new `OutRestResponse`.
    pub fn new(value: T, pages: u8) -> Self {
        Self { value, pages }
    }
}

pub fn calc_pages(pagination: Pagination, config: PaginationConfig) -> Result<u8, String> {
    let pagination_total = pagination
        .total
        .parse::<u32>()
        .map_err(|_| format!("Cannot parse pagination total, '{}'", pagination.total))?;

    if pagination_total < config.get_limit().into() {
        Ok(1)
    } else if config.get_offset() >= pagination_total {
        Err("There is no error. And this page doesn't have any data to show you.".to_string())
    } else {
        let num_of_full_pages = pagination_total / config.get_limit() as u32;

        let num_of_pages = if pagination_total % config.get_limit() as u32 > 0 {
            num_of_full_pages + 1
        } else {
            num_of_full_pages
        };

        Ok(num_of_pages as u8)
    }
}

pub fn extract_chain(chain: &str, chains: Data<State>) -> Result<Chain, TNRAppError> {
    chains.get(chain).map_err(|_| TNRAppError {
        message: Some(format!("Chain could not found {}", chain)),
        error_type: TNRAppErrorType::MessageError,
    })
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ChainAmountItem {
    pub amount: TnrDecimal,
    pub ticker: String,
}

impl Default for ChainAmountItem {
    fn default() -> Self {
        Self {
            amount: TnrDecimal::ZERO,
            ticker: String::from(""),
        }
    }
}

impl ChainAmountItem {
    pub async fn new(amount: TnrDecimal, ticker: String, chain: Chain) -> Self {
        match ChainAmountItemBuilder::new()
            .chain(chain)
            .amount(amount)
            .token(ticker.clone())
            .build()
            .await
        {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Cannot build ChainAmountItem {}", e);
                Self { amount, ticker }
            }
        }
    }

    pub fn sync_with_ticker(amount: TnrDecimal, ticker: String) -> Self {
        Self { amount, ticker }
    }
}
