use crate::chain::Chain;

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

    pub fn calc_amount_f64_to_f64(&self, amount: f64) -> f64 {
        amount / (self.config.decimals_pow as f64 * 10000.0)
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
            .unwrap_or(&validator_delegator_shares)
            .parse::<u128>()
            .map_err(|_| format!("Cannot parse delegator shares, {}.", validator_delegator_shares))?;

        Ok(self.calc_amount_u128_to_f64(formatted))
    }

    pub fn generate_heartbeat_id(&self, sender_address: String, height: u64) -> String {
        format!("{}_{}", sender_address, height)
    }
}
