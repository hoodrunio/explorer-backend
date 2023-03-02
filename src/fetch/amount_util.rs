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
                    let cosmos_decimal = 6;
                    let mut ticker_result = String::from("Unknown");
                    let mut amount_result = chain.calc_tnr_decimal_amount(amount, None);
                    if let Ok(ibc_denom) = chain.convert_to_ibc_denom(&ticker) {
                        ticker_result = ibc_denom;
                        amount_result = chain.calc_tnr_decimal_amount(amount, Some(cosmos_decimal));
                    };

                    amount = amount_result;
                    ticker = ticker_result;
                }
            };
        };

        Ok(ChainAmountItem { amount, ticker })
    }
}

pub type TnrDecimal = Decimal;
