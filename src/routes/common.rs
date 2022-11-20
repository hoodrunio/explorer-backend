use crate::fetch::others::{Pagination, PaginationConfig};
use serde::{Deserialize, Serialize};

/// The query params that has `page` param.
#[derive(Deserialize)]
pub struct QueryParams {
    /// Page. Indexing starts at 1.\
    /// The value is `1/ONE`, if it is not specified. \
    /// Example values are: 1, 2, 3, 4, etc
    pub page: Option<u8>,
}

#[derive(Serialize)]
pub struct OutRestResponse<T> {
    pub value: T,
    /// The count of the pages.
    pub pages: u8,
}

impl<T> OutRestResponse<T> {
    /// Tries to create a new `OutRestResponse`.
    pub fn new(value: T, pages: u8) -> Result<Self, String> {
        Ok(Self { value, pages })
    }
}

pub fn calc_pages(pagination: Pagination, config: PaginationConfig) -> Result<u8, String> {
    let pagination_total = pagination
        .total
        .parse::<u32>()
        .or_else(|_| Err(format!("Cannot parse pagination total, '{}'", pagination.total)))?;

    if pagination_total < config.get_limit().into() {
        Ok(1)
    } else if config.get_offset() >= pagination_total {
        Err(format!("There is no error. And this page doesn't have any data to show you."))
    } else {
        Ok((pagination_total / config.get_limit() as u32) as u8)
    }
}
