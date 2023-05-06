use crate::routes::{PaginationData, PaginationDirection, TNRAppSuccessResponse};
use mongodb_cursor_pagination::{FindResult, PageInfo};
use serde::{Deserialize, Serialize};
use crate::fetch::PaginationResponse;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListDbResult<T> {
    /// Array of validators.
    pub data: Vec<T>,
    /// Pagination.
    pub pagination: PaginationData,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PaginatedListResult<T> {
    pub data: Vec<T>,
    pub pagination: PaginationResponse,
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

impl<T> From<ListDbResult<T>> for TNRAppSuccessResponse<Vec<T>> {
    fn from(value: ListDbResult<T>) -> Self {
        TNRAppSuccessResponse {
            data: value.data,
            pagination: Some(value.pagination),
        }
    }
}
