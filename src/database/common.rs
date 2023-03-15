use mongodb_cursor_pagination::{FindResult, PageInfo};
use serde::{Deserialize, Serialize};
use crate::routes::{PaginationData, PaginationDirection, TNRAppSuccessResponse};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListDbResult<T> {
    /// Array of validators.
    pub data: Vec<T>,
    /// Pagination.
    pub pagination: PaginationData,
}

impl<T> From<FindResult<T>> for ListDbResult<T> {
    fn from(value: FindResult<T>) -> Self {
        let pagination = PaginationData {
            cursor: value.page_info.next_cursor,
            limit: Some(value.items.len() as u64),
            direction: Some(PaginationDirection::Next),
            ..Default::default()
        };

        Self {
            data: value.items,
            pagination,
        }
    }
}

impl From<PageInfo> for PaginationData {
    fn from(value: PageInfo) -> Self {
        Self {
            cursor: value.next_cursor,
            direction: Some(PaginationDirection::Next),
            ..Default::default()
        }
    }
}

impl<T> ListDbResult<T> {
    pub fn new(data: Vec<T>, page_info: PageInfo, total: u64) -> Self {
        Self {
            data,
            pagination: page_info.into()
        }
    }
}