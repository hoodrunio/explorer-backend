use rust_decimal::prelude::FromPrimitive;
use sha2::{Digest, Sha256};

use crate::chain::Chain;

use crate::routes::ChainAmountItem;
use hex::encode as to_hex;

use super::amount_util::TnrDecimal;

impl Chain {
    pub fn calc_amount_u128_to_u64(&self, amount: u128) -> u64 {
        (amount / (self.config.decimals_pow * 10000) as u128) as u64
    }

    pub fn calc_amount_u128_to_f64(&self, amount: u128) -> f64 {
        (amount / (self.config.decimals_pow) as u128) as f64
    }

    pub fn _calc_amount_f64_to_u64(&self, amount: f64) -> u64 {
        amount as u64 / (self.config.decimals_pow * 10000)
    }

    // pub fn calc_amount_f64_to_f64(&self, amount: f64) -> f64 {
    //     amount / (self.config.decimals_pow as f64 * 10000.0)
    // }

    pub fn calc_tnr_decimal_amount(&self, amount: TnrDecimal, decimal: Option<i64>) -> TnrDecimal {
        let main_decimal = match decimal {
            Some(res) => 10_u64.pow(res as u32) as f64,
            None => self.config.decimals_pow as f64 * 10000.0,
        };
        let other = TnrDecimal::from_f64(main_decimal).unwrap_or(TnrDecimal::ONE);
        amount.checked_div(other).unwrap_or(TnrDecimal::ZERO)
    }

    /// Returns the amount parsed.
    /// # Usage
    /// ```rs
    /// // 0.030437
    /// let amount = self.string_amount_parser("30437.0000").await;
    /// ```
    pub async fn string_amount_parser(&self, string_amount: String, ticker: Option<String>) -> Result<ChainAmountItem, String> {
        let float_amount = TnrDecimal::from_str_exact(&string_amount).unwrap_or(TnrDecimal::ZERO);

        let ticker = match ticker {
            None => self.config.main_symbol.clone(),
            Some(some) => some,
        };
        Ok(ChainAmountItem::new(float_amount, ticker, self.clone()).await)
    }

    //https://tutorials.cosmos.network/tutorials/6-ibc-dev/
    //Check if denom is ibc denom path with transfer/channel-{{channel_id}}/denom
    pub fn is_ibc_denom_path(&self, denom: &str) -> bool {
        let split = denom.split('/').collect::<Vec<&str>>();
        split.len() == 3 && split[0] == "transfer" && split[1].starts_with("channel")
    }

    //```
    //https://tutorials.cosmos.network/tutorials/6-ibc-dev/
    //Converts ibc transfer path to ibc denom format if given paramters valid
    //Returns ibc/{{converted_value}}
    ///```
    pub fn convert_to_ibc_denom(&self, path: &String) -> Result<String, String> {
        if self.is_ibc_denom_path(path) {
            let mut hasher = Sha256::new();
            hasher.update(path.as_bytes());
            let result = to_hex(hasher.finalize()).to_uppercase();
            return Ok(format!("IBC/{}", result));
        };

        Err(format!("Not an IBC denom path: {}", path))
    }

    /// Returns the amount parsed.
    /// # Usage
    /// ```rs
    /// // 0.030437
    /// let amount = axelar.get_amount("30437uaxl");
    /// ```
    pub fn _get_amount(&self, amount: &str) -> f64 {
        if amount.len() > self.config.main_denom.len() {
            let str_amount = &amount[..amount.len() - self.config.main_denom.len()];

            let amount: u128 = match str_amount.parse() {
                Ok(amount) => amount,
                _ => return 0.00,
            };

            self.calc_amount_u128_to_f64(amount)
        } else {
            0.00
        }
    }

    pub fn convert_valoper_to_self_delegate_address(&self, valoper_addr: &str) -> Option<String> {
        bech32::encode(&self.config.base_prefix, bech32::decode(valoper_addr).ok()?.1, bech32::Variant::Bech32).ok()
    }

    pub fn format_delegator_share(&self, validator_delegator_shares: &str) -> Result<f64, String> {
        let formatted = validator_delegator_shares
            .split_once('.')
            .map(|(pri, _)| pri)
            .unwrap_or(validator_delegator_shares)
            .parse::<u128>()
            .map_err(|_| format!("Cannot parse delegator shares, {}.", validator_delegator_shares))?;

        Ok(self.calc_amount_u128_to_f64(formatted))
    }

    pub fn generate_heartbeat_id(&self, sender_address: String, height: u64) -> String {
        format!("{}_{}", sender_address, height)
    }
}
