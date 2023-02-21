use mongodb_cursor_pagination::FindResult;
use serde::{Deserialize, Serialize};
use crate::routes::{PaginationData, PaginationDirection};

#[derive(Deserialize, Serialize, Debug)]
pub struct ListDbResult<T> {
    /// Array of validators.
    pub list: Vec<T>,
    /// Pagination.
    pub pagination: PaginationData,
}

impl<T> From<FindResult<T>> for ListDbResult<T> {
    fn from(value: FindResult<T>) -> Self {
        let pagination = PaginationData {
            cursor: value.page_info.next_cursor,
            limit: value.items.len() as u64,
            direction: Some(PaginationDirection::Next),
            ..Default::default()
        };

        Self {
            list: value.items,
            pagination,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PaginationDb {
    pub page: u16,
    pub total: u16,
}