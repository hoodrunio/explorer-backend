use serde::de::DeserializeOwned;

use super::{
    blocks::{BlockResp, BlockchainResp},
    others::*,
    requests::RPCResponse,
    transactions::{TxResp, TxsResp},
};

/// The trait that provides methods for common operation types.
#[async_trait::async_trait]
pub trait Chain<'a>: Sync {
    /*
    ------------------------------------------------------
       METHODS THAT NEED TO BE IMPLEMENTED ESPECIALLY.
       OTHERS WILL BE AUTOMATICALLY IMPLEMENTED.
    ------------------------------------------------------
    */

    /// Initializes the chain.
    fn new(client: &'a reqwest::Client) -> Self;

    /// Returns the name of the chain.
    fn name(&self) -> &'static str;

    /// Returns the logo URL of the chain.
    fn logo(&self) -> &'static str;

    /// Returns the base prefix of the chain.
    fn base_prefix(&self) -> &'static str;

    /// Returns REST API URL of the chain.
    fn rest_url(&self) -> &'static str;

    /// Returns REST API URL of the chain.
    fn rpc_url(&self) -> &'static str;

    /// Returns Cosmos SDK version of the chain.
    fn sdk_version(&self) -> usize;

    /// Returns the decimals of native coin of the chain.
    fn decimals(&self) -> usize;

    /// Returns `reqwest::Client` of the chain.
    fn client(&self) -> &reqwest::Client;

    /*
    --------------------------------------
        Methods for prefix operations.
    --------------------------------------
    */

    /// Returns the account prefix of the chain.
    fn account_prefix(&self) -> String {
        format!("{}", self.base_prefix())
    }

    /// Returns the valoper prefix of the chain.
    fn valoper_prefix(&self) -> String {
        format!("{}valoper", self.base_prefix())
    }

    /// Returns the cons prefix of the chain.
    fn cons_prefix(&self) -> String {
        format!("{}valcons", self.base_prefix())
    }

    /*
    ---------------------------------------
        METHODS FOR REQUEST OPERATIONS.
    ---------------------------------------
    */

    /// Makes an RPC request.
    async fn rpc_request<T: DeserializeOwned>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
        let url = format!("{}{}", self.rpc_url(), path);

        match self.client().get(url).query(query).send().await {
            Ok(res) => match res.json::<RPCResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RPCResponse::Success(res) => Ok(res.result),
                    RPCResponse::Error(res) => Err(res.error.data),
                },
                Err(_) => Err("Cannot parse JSON.".to_string()),
            },
            Err(_) => Err("Unsuccessful request.".to_string()),
        }
    }

    /// Makes REST API request.
    async fn rest_api_request<T: DeserializeOwned>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String> {
        let url = format!("{}{}", self.rest_url(), path);

        match self.client().get(url).query(query).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    match res.json().await {
                        Ok(res_json) => Ok(res_json),
                        Err(error) => {
                            println!("{:#?}", error);
                            Err("Cannot parse JSON error response.".to_string())
                        }
                    }
                } else {
                    match res.json().await {
                        Ok(res_json) => Err(res_json),
                        Err(error) => {
                            println!("{:#?}", error);
                            Err("Cannot parse JSON error response.".to_string())
                        }
                    }
                }
            }
            Err(_) => Err("Unsuccessful request.".to_string()),
        }
    }

    /*
    -------------------------------------
        METHODS FOR BLOCK OPERATIONS.
    -------------------------------------
    */

    /// Returns the block at given height. Returns the latest block, if no height is given.
    async fn get_block_by_height(&self, height: Option<usize>) -> Result<BlockResp, String> {
        let mut query = vec![];

        let height = height.and_then(|height| Some(height.to_string()));

        if let Some(height) = height {
            query.push(("height", height))
        }

        self.rpc_request("/block", &query).await
    }

    /// Returns the block with given hash.
    /// # Usage
    /// ```rs
    /// let block = chain.get_block_by_hash("14b6bb26cf30a559ae3ad18b0e3640bc3fd819b1182830d359969e02bab0f633").await;
    /// ```
    async fn get_block_by_hash(&self, hash: &str) -> Result<BlockResp, String> {
        let mut query = vec![];

        let hash = if hash.starts_with("0x") {
            hash.to_string()
        } else {
            format!("0x{}", hash)
        };

        query.push(("hash", hash));

        self.rpc_request("/block_by_hash", &query).await
    }

    /// Returns transaction by given hash. Hash should start with `0x`.
    async fn get_blockchain(&self, min_height: u64, max_height: u64) -> Result<BlockchainResp, String> {
        let mut query = vec![];

        query.push(("minHeight", min_height.to_string()));
        query.push(("maxHeight", max_height.to_string()));

        self.rpc_request("/blockchain", &query).await
    }

    /*
    -------------------------------------------
        METHODS FOR TRANSACTION OPERATIONS.
    -------------------------------------------
    */

    /// Returns transaction by given hash.
    async fn get_tx_by_hash(&self, hash: &str) -> Result<TxResp, String> {
        let path = format!("/cosmos/tx/v1beta1/txs/{hash}");

        self.rest_api_request::<TxResp>(&path, &[]).await
    }

    /// Returns transactions with given sender.
    async fn get_txs_by_sender(&self, sender_address: &str, pagination_config: PaginationConfig) -> Result<TxsResp, String> {
        let mut query = vec![];

        query.push(("events", format!("message.sender='{}'", sender_address)));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        self.rest_api_request("/cosmos/tx/v1beta1/txs", &query).await
    }

    /// Returns transactions with given recipient.
    async fn get_txs_by_recipient(
        &self,
        recipient_address: &str,
        pagination_config: PaginationConfig,
    ) -> Result<TxsResp, String> {
        let mut query = vec![];

        query.push(("events", format!("message.recipient='{}'", recipient_address)));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        self.rest_api_request("/cosmos/tx/v1beta1/txs", &query).await
    }

    /// Returns transactions at given height.
    async fn get_txs_by_height(&self, block_height: u64, pagination_config: PaginationConfig) -> Result<TxsResp, String> {
        let mut query = vec![];

        query.push(("events", format!("tx.height={}", block_height)));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        self.rest_api_request("/cosmos/tx/v1beta1/txs", &query).await
    }

    /*
    --------------------------------------------------
        METHODS FOR GETTING PARAMETERS OPERATIONS.
    --------------------------------------------------
    */

    /// Returns the staking parameters.
    async fn get_staking_params(&self) -> Result<ParamsResp<StakingParams>, String> {
        self.rest_api_request("/cosmos/staking/v1beta1/params", &[]).await
    }

    /// Returns the slashing parameters of the chain.
    async fn get_slashing_params(&self) -> Option<ParamsResp<SlashingParams>> {
        self.rest_api_request("/cosmos/slashing/v1beta1/params", &[])
            .await
            .unwrap_or(None)
    }

    /// Returns the voting parameters.
    async fn get_voting_params(&self) -> Result<VotingParams, String> {
        self.rest_api_request::<VotingParamsResp>("/cosmos/gov/v1beta1/params/voting", &[])
            .await
            .and_then(|res| Ok(res.voting_params))
    }

    /// Returns the deposit parameters.
    async fn get_deposit_params(&self) -> Result<DepositParams, String> {
        self.rest_api_request::<DepositParamsResp>("/cosmos/gov/v1beta1/params/deposit", &[])
            .await
            .and_then(|res| Ok(res.deposit_params))
    }

    /// Returns the tallying parameters.
    async fn get_tally_params(&self) -> Result<TallyParams, String> {
        self.rest_api_request::<TallyingParamsResp>("/cosmos/gov/v1beta1/params/tallying", &[])
            .await
            .and_then(|res| Ok(res.tally_params))
    }

    /*
    -----------------------------------------
        METHODS FOR VALIDATOR OPERATIONS.
    -----------------------------------------
    */

    /// Returns validator by given validator address.
    async fn get_validator(&self, validator_addr: &str) -> Result<ValidatorResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/validators/{validator_addr}");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns all the validators by given delegator address.
    async fn get_validators_by_delegator(
        &self,
        delegator_addr: &str,
        pagination_config: PaginationConfig,
    ) -> Result<ValidatorsResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/validators");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request(&path, &query).await
    }

    /// Returns accumulated commission of given validator.
    async fn get_validator_commission(&self, validator_addr: &str) -> Result<ValidatorCommisionResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/validators/{validator_addr}/commission");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns rewards of given validator.
    async fn get_validator_rewards(&self, validator_addr: &str) -> Result<ValidatorRewardsResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/validators/{validator_addr}/outstanding_rewards");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the list of validators with bonded status.
    async fn get_validators_bonded(&self, pagination_config: PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_BONDED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the list of validators with unbonded status.
    async fn get_validators_unbonded(&self, pagination_config: PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNBONDED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the list of validators with unbonding status.
    async fn get_validators_unbonding(&self, pagination_config: PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNBONDING".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }
    /// Returns the list of validators with unspecified status.
    async fn get_validators_unspecified(&self, pagination_config: PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNSPECIFIED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /*
    -----------------------------------------
        METHODS FOR DELEGATOR OPERATIONS.
    -----------------------------------------
    */

    /// Returns the withdraw address by given delegator address.
    async fn get_delegator_withdraw_address(&self, delegator_addr: &str) -> Result<WithdrawAddressResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/delegators/{delegator_addr}/withdraw_address");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the rewards of given delegator address.
    async fn get_delegator_rewards(&self, delegator_addr: &str) -> Result<DelegatorRewardsResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/delegators/{delegator_addr}/rewards");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns validator information by given delegator validator pair.
    async fn get_delegator_validator_pair_info(
        &self,
        delegator_addr: &str,
        validator_addr: &str,
    ) -> Result<ValidatorResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/validators/{validator_addr}");

        self.rest_api_request(&path, &[]).await
    }

    /*
    ----------------------------------------------------
        METHODS FOR SUPPLY AND INFLATION OPERATIONS.
    ----------------------------------------------------
    */

    /// Returns the total supply of all tokens.
    async fn get_supply_of_all_tokens(&self, pagination_config: PaginationConfig) -> Result<SupplyOfAllTokensResp, String> {
        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/bank/v1beta1/supply", &query).await
    }

    /// Returns the supply of given token.
    async fn get_supply_by_denom(&self, denom: &str) -> Result<SupplyByDenomResp, String> {
        let path = format!("/cosmos/bank/v1beta1/supply/{denom}");
        self.rest_api_request(&path, &[]).await
    }

    /// Returns the minting inflation rate of native coin of the chain.
    async fn get_inflation_rate(&self) -> f64 {
        if self.name() == "evmos" {
            self.rest_api_request::<MintingInflationRateResp>("/evmos/inflation/v1/inflation_rate", &[])
                .await
                .and_then(|res| Ok(res.inflation_rate.parse::<f64>().unwrap_or(0.0) / 100.0))
        } else if self.name() == "echelon" {
            self.rest_api_request::<MintingInflationRateResp>("/echelon/inflation/v1/inflation_rate", &[])
                .await
                .and_then(|res| Ok(res.inflation_rate.parse::<f64>().unwrap_or(0.0) / 100.0))
        } else {
            self.rest_api_request::<MintingInflationResp>("/cosmos/mint/v1beta1/inflation", &[])
                .await
                .and_then(|res| Ok(res.inflation.parse::<f64>().unwrap_or(0.0)))
        }
        .unwrap_or(0.0)
    }

    /*
    ----------------------------------------
        METHODS FOR PROPOSAL OPERATIONS.
    ----------------------------------------
    */

    /// Returns all the proposals in voting period.
    async fn get_proposals_by_status(&self, status: &str, pagination_config: PaginationConfig) -> Result<ProposalsResp, String> {
        let mut query = vec![];

        query.push(("proposal_status", status.to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/gov/v1beta1/proposals", &query).await
    }

    /// Returns all the proposals unspecified.
    async fn get_proposals_unspecified(&self, pagination_config: PaginationConfig) -> Result<ProposalsResp, String> {
        self.get_proposals_by_status("1", pagination_config).await
    }

    /// Returns all the proposals in voting period.
    async fn get_proposals_in_voting_period(&self, pagination_config: PaginationConfig) -> Result<ProposalsResp, String> {
        self.get_proposals_by_status("2", pagination_config).await
    }

    /// Returns all the proposals passed.
    async fn get_proposals_passed(&self, pagination_config: PaginationConfig) -> Result<ProposalsResp, String> {
        self.get_proposals_by_status("3", pagination_config).await
    }

    /// Returns all the proposals rejected.
    async fn get_proposals_rejected(&self, pagination_config: PaginationConfig) -> Result<ProposalsResp, String> {
        self.get_proposals_by_status("4", pagination_config).await
    }

    /// Returns all the proposals failed.
    async fn get_proposals_failed(&self, pagination_config: PaginationConfig) -> Result<ProposalsResp, String> {
        self.get_proposals_by_status("5", pagination_config).await
    }

    /// Returns the details of given proposal.
    async fn get_proposal_details(&self, proposal_id: u64) -> Result<ProposalsDetailsResp, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the deposits of given proposal.
    async fn get_proposal_deposits(
        &self,
        proposal_id: u64,
        pagination_config: PaginationConfig,
    ) -> Result<ProposalDepositsResp, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/deposits");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request(&path, &query).await
    }

    /// Returns the deposit of given proposal by given depositor.
    async fn get_proposal_deposit_by_depositor(
        &self,
        proposal_id: u64,
        depositor: &str,
    ) -> Result<ProposalDepositByDepositorResp, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/deposits/{depositor}");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the tally of given proposal.
    async fn get_proposal_tally(&self, proposal_id: u64) -> Result<ProposalTallyResp, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/tally");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the votes of given proposal.
    async fn get_proposal_votes(
        &self,
        proposal_id: u64,
        pagination_config: PaginationConfig,
    ) -> Result<ProposalVotesResp, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/votes");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request(&path, &query).await
    }

    /// Returns the vote of given proposal by given voter.
    async fn get_proposal_vote_by_voter(&self, proposal_id: u64, voter: &str) -> Result<ProposalVoteByVoterResp, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/votes/{voter}");

        self.rest_api_request(&path, &[]).await
    }

    /*
    ------------------------------------------
        METHODS FOR DELEGATION OPERATIONS.
    ------------------------------------------
    */

    /// Returns the delegations of given address.
    async fn get_delegations(
        &self,
        delegator_addr: &str,
        pagination_config: PaginationConfig,
    ) -> Result<DelagationsResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegations/{delegator_addr}");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request(&path, &query).await
    }

    /// Returns the redelegations of given address.
    async fn get_redelegations(
        &self,
        delegator_addr: &str,
        pagination_config: PaginationConfig,
    ) -> Result<RedelagationsResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/redelegations");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request(&path, &query).await
    }

    /// Returns the unbonding delegations of given address.
    async fn get_delegations_unbonding(
        &self,
        delegator_addr: &str,
        pagination_config: PaginationConfig,
    ) -> Result<UnbondingDelegationResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/unbonding_delegations");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request(&path, &query).await
    }

    /*
    --------------------------------------------
        METHODS FOR STAKING POOL OPERATIONS.
    --------------------------------------------
    */

    /// Returns staking pool information.
    async fn get_staking_pool(&self) -> Result<StakingPoolResp, String> {
        self.rest_api_request("/cosmos/staking/v1beta1/pool", &[]).await
    }

    /*
    ---------------------------------------
        METHODS FOR SIGNING OPERATIONS.
    ---------------------------------------
    */

    /// Returns the signing info by given cons address.
    async fn get_signing_info(&self, cons_addr: &str) -> Result<SigningInfoResp, String> {
        let path = format!("/cosmos/slashing/v1beta1/signing_infos/{cons_addr}");

        self.rest_api_request(&path, &[]).await
    }
}
