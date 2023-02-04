use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ListDbResult<T> {
    /// Array of validators.
    pub list: Vec<T>,
    /// Pagination.
    pub pagination: PaginationDb,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PaginationDb {
    pub page: u16,
    pub total: u16,
}