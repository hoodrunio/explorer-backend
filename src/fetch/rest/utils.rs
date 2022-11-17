use super::others::{DenomAmount, Pagination, PaginationConfig};
use crate::chain::Chain;

impl Chain {
    pub fn valoper_addr(&self, addr: &str) -> String {
        format!("{}{}", self.inner.valoper_prefix, addr)
    }

    pub fn cons_addr(&self, addr: &str) -> String {
        format!("{}{}", self.inner.cons_prefix, addr)
    }
}
