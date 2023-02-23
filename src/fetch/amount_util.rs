use rust_decimal::Decimal;

use crate::chain::Chain;
use crate::routes::ChainAmountItem;

pub struct ChainAmountItemBuilder {
    pub amount: TnrDecimal,
    pub ticker: String,
    pub chain: Option<Chain>,
}

impl ChainAmountItemBuilder {
    pub fn new() -> Self {
        Self {
            amount: TnrDecimal::ZERO,
            ticker: String::from(""),
            chain: None,
        }
    }

    pub fn amount(&mut self, amount: TnrDecimal) -> &mut Self {
        self.amount = amount;
        self
    }

    pub fn token(&mut self, token: String) -> &mut Self {
        self.ticker = token;
        self
    }

    pub fn chain(&mut self, chain: Chain) -> &mut Self {
        self.chain = Some(chain);
        self
    }

    pub async fn build(&self) -> Result<ChainAmountItem, String> {
        let mut amount = self.amount;
        let mut ticker = self.ticker.clone();

        if let Some(chain) = &self.chain {
            let assets = chain.cosmos_assets().await?.assets;
            match assets.into_iter().find(|asset| asset.symbol == ticker || asset.denom == ticker) {
                Some(asset) => {
                    amount = chain.calc_tnr_decimal_amount(amount, Some(asset.decimals));
                    ticker = asset.symbol;
                }
                None => {
                    amount = chain.calc_tnr_decimal_amount(amount, None);
                    ticker = String::from("Unknown");
                }
            };
        };

        Ok(ChainAmountItem { amount, ticker })
    }
}

pub type TnrDecimal = Decimal;
