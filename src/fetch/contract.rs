use crate::chain::Chain;
use crate::database::ContractData;

impl Chain {
    /// Returns transactions from db.
    pub async fn get_contract_by_hash(&self, contract_address: &str) -> Result<ContractData, String> {
        let contract_data = self.database.find_contract_data_by_contract_address(contract_address).await?;

        Ok(contract_data)
    }
}