use mongodb_cursor_pagination::FindResult;
use serde::{Deserialize, Serialize};
use crate::routes::PaginationData;
use crate::routes::PaginationDirection::Next;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListDbResult<T> {
    /// Array of validators.
    pub list: Vec<T>,
    /// Pagination.
    pub pagination: PaginationData,
}

impl<T> From<FindResult<T>> for ListDbResult<T> {
    fn from(value: FindResult<T>) -> Self {
        let len = value.items.len() as u64;
        Self {
            list: value.items,
            pagination: PaginationData {
                cursor: value.page_info.next_cursor,
                offset: None,
                limit: Some(len),
                direction: Some(Next),
            },
        }
    }
}


#[derive(Deserialize, Serialize, Debug)]
pub struct PaginationDb {
    pub page: u16,
    pub total: u16,
}