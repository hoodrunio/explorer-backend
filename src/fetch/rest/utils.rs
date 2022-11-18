use std::ops::Div;

use super::others::{DenomAmount, Pagination, PaginationConfig};
use crate::chain::Chain;

impl Chain {
    pub fn valoper_addr(&self, addr: &str) -> String {
        format!("{}{}", self.inner.valoper_prefix, addr)
    }

    pub fn cons_addr(&self, addr: &str) -> String {
        format!("{}{}", self.inner.cons_prefix, addr)
    }

    pub fn base_to_valoper(&self, base_addr: &str) -> Result<String, String> {
        if base_addr.starts_with(self.inner.base_prefix) {
            Ok(format!("{}{}", self.inner.valoper_prefix, &base_addr[self.inner.base_prefix.len()..]))
        } else {
            Err(format!("Address is mistaken, '{}'.", base_addr))
        }
    }

    pub fn calc_amount_u128_to_u64(&self, amount: u128) -> u64 {
        (amount / (self.inner.decimals_pow * 10000) as u128) as u64
    }

    pub fn calc_amount_u128_to_f64(&self, amount: u128) -> f64 {
        (amount / (self.inner.decimals_pow) as u128) as f64 / 10000.0
    }

    pub fn calc_amount_f64_to_u64(&self, amount: f64) -> u64 {
        amount as u64 / (self.inner.decimals_pow * 10000)
    }

    pub fn calc_amount_f64_to_f64(&self, amount: f64) -> f64 {
        amount / (self.inner.decimals_pow as f64 * 10000.0)
    }
}
