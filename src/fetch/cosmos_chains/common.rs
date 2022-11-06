use async_trait::async_trait;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};
/// The struct that stores important URLs of a chain.
pub struct ChainUrls {
    /// The REST API URL of the chain.
    pub rest_api: &'static str,
    /// The RPC URL of the chain.
    pub rpc: &'static str,
}

/// The trait that provides methods for common operation types.
#[async_trait]
pub trait Chain
where
    Self: Sync,
{
    /// Returns the name of the chain.
    fn name(&self) -> &'static str;

    /// Returns the `ChainUrls` of the chain.
    fn urls(&self) -> &ChainUrls;

    /// Returns Cosmos SDK version of the chain.
    fn sdk_version(&self) -> usize;

    /// Returns `reqwest::Client` of the chain.
    fn client(&self) -> &Client;

    /// Makes an RPC request.
    async fn rpc_request<T>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.urls().rpc, path);

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
    async fn rest_api_request<T>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.urls().rest_api, path);

        match self.client().get(url).query(query).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    match res.json().await {
                        Ok(res_json) => Ok(res_json),
                        Err(_) => Err("Cannot parse JSON error response.".to_string()),
                    }
                } else {
                    match res.json().await {
                        Ok(res_json) => Err(res_json),
                        Err(_) => Err("Cannot parse JSON error response.".to_string()),
                    }
                }
            }
            Err(_) => Err("Unsuccessful request.".to_string()),
        }
    }

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
    async fn get_block_by_hash(&self, hash: &str) -> Result<BlockResp, String> {
        let mut query = vec![];

        query.push(("hash", hash.to_string()));

        self.rpc_request("/block_by_hash", &query).await
    }

    /// Returns transaction by given hash. Hash should start with `0x`.
    async fn get_tx_by_hash(&self, hash: &str) -> Result<TransactionResp, String> {
        let mut query = vec![];

        query.push(("hash", hash.to_string()));

        self.rpc_request("/tx", &query).await
    }

    /// Returns transaction by given hash. Hash should start with `0x`.
    async fn get_blockchain(&self, min_height: u64, max_height: u64) -> Result<BlockchainResp, String> {
        let mut query = vec![];

        query.push(("minHeight", min_height.to_string()));
        query.push(("maxHeight", max_height.to_string()));

        self.rpc_request("/blockchain", &query).await
    }

    /// Returns transactions with given sender.
    async fn get_txs_by_sender(&self, sender_address: &str, pagination_config: PaginationConfig) -> Result<TxsResp, String> {
        let mut query = vec![];

        query.push(("events", format!("message.sender='{}'", sender_address)));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        self.rest_api_request("/cosmos/tx/v1beta1/txs", &query).await
    }

    /// Returns transactions with given recipient.
    async fn get_txs_by_recipient(
        &self,
        recipient_address: &str,
        pagination_config: &PaginationConfig,
    ) -> Result<TxsResp, String> {
        let mut query = vec![];

        query.push(("events", format!("message.recipient='{}'", recipient_address)));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        self.rest_api_request("/cosmos/tx/v1beta1/txs", &query).await
    }

    /// Returns transactions at given height.
    async fn get_txs_by_height(&self, block_height: u64, pagination_config: &PaginationConfig) -> Result<TxsResp, String> {
        let mut query = vec![];

        query.push(("events", format!("tx.height={}", block_height)));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        self.rest_api_request("/cosmos/tx/v1beta1/txs", &query).await
    }

    /// Returns validator by given validator address.
    async fn get_validator(&self, validator_addr: &str) -> Result<ValidatorResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/validators/{validator_addr}");

        self.rest_api_request(&path, &[]).await
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

    /// Returns the total supply of all tokens.
    async fn get_supply_of_all_tokens(&self, pagination_config: &PaginationConfig) -> Result<SupplyOfAllTokensResp, String> {
        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/bank/v1beta1/supply", &query).await
    }

    /// Returns the supply of given token.
    async fn get_supply_by_denom(&self, denom: &str) -> Result<SupplyByDenomResp, String> {
        let path = format!("/cosmos/bank/v1beta1/supply/{denom}");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns staking pool information.
    async fn get_staking_pool(&self) -> Result<StakingPoolResp, String> {
        self.rest_api_request("/cosmos/staking/v1beta1/pool", &[]).await
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

    /// Returns the staking parameters.
    async fn get_staking_params(&self) -> Result<ParamsResp<StakingParams>, String> {
        self.rest_api_request("/cosmos/staking/v1beta1/params", &[]).await
    }

    /// Returns the list of validators with bonded status.
    async fn get_validator_list_bonded(&self, pagination_config: &PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_BONDED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the list of validators with unbonded status.
    async fn get_validator_list_unbonded(&self, pagination_config: &PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNBONDED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the list of validators with unbonding status.
    async fn get_validator_list_unbonding(&self, pagination_config: &PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNBONDING".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }
    /// Returns the list of validators with unspecified status.
    async fn get_validator_list_unspecified(&self, pagination_config: &PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNSPECIFIED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the slashing parameters of the chain.
    async fn get_slashing_params(&self) -> Option<ParamsResp<SlashingParams>> {
        self.rest_api_request("/cosmos/slashing/v1beta1/params", &[])
            .await
            .unwrap_or(None)
    }

    // Returns the mint parameters of the chain.
    /* async fn get_mint_params(&self) -> Option<MintParams> {
        if self.name() == "evmos" {
            self.rest_api_request::<ParamsResp<InflationParams>>("/evmos/inflation/v1/params", &[]).await.and_then(|a|)
        } else if self.name() == "echelon" {
            self.rest_api_request::<ParamsResp<InflationParams>>("/echelon/inflation/v1/params", &[]).await
        } else {
            self.rest_api_request::<ParamsResp<MintParams>>("/cosmos/mint/v1beta1/params", &[])
                .await
                .ok()
                .and_then(|res| Some(res.params))
        }
        .unwrap_or(None)
    }
    */

    /// Returns all the proposals in voting period.
    async fn get_proposals_in_voting_period(&self, pagination_config: &PaginationConfig) -> Result<ProposalsResp, String> {
        let mut query = vec![];

        query.push(("proposal_status", "2".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/gov/v1beta1/proposals", &query).await
    }

    /// Returns all the proposals passed.
    async fn get_proposals_passed(&self, pagination_config: &PaginationConfig) -> Result<ProposalsResp, String> {
        let mut query = vec![];

        query.push(("proposal_status", "3".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/gov/v1beta1/proposals", &query).await
    }

    /// Returns all the proposals rejected.
    async fn get_proposals_rejected(&self, pagination_config: &PaginationConfig) -> Result<ProposalsResp, String> {
        let mut query = vec![];

        query.push(("proposal_status", "4".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/gov/v1beta1/proposals", &query).await
    }

    /// Returns all the proposals failed.
    async fn get_proposals_failed(&self, pagination_config: &PaginationConfig) -> Result<ProposalsResp, String> {
        let mut query = vec![];

        query.push(("proposal_status", "5".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/gov/v1beta1/proposals", &query).await
    }

    /// Returns all the proposals unspecified.
    async fn get_proposals_unspecified(&self, pagination_config: &PaginationConfig) -> Result<ProposalsResp, String> {
        let mut query = vec![];

        query.push(("proposal_status", "1".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/gov/v1beta1/proposals", &query).await
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
        pagination_config: &PaginationConfig,
    ) -> Result<ProposalDepositsResp, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/deposits");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

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
        pagination_config: &PaginationConfig,
    ) -> Result<ProposalVotesResp, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/votes");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request(&path, &query).await
    }

    /// Returns the vote of given proposal by given voter.
    async fn get_proposal_vote_by_voter(&self, proposal_id: u64, voter: &str) -> Result<ProposalVoteByVoterResp, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/votes/{voter}");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the delegations of given address.
    async fn get_delegations(
        &self,
        delegator_addr: &str,
        pagination_config: &PaginationConfig,
    ) -> Result<DelagationsResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegations/{delegator_addr}");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request(&path, &query).await
    }

    /// Returns the redelegations of given address.
    async fn get_delegator_redelegations(
        &self,
        delegator_addr: &str,
        pagination_config: &PaginationConfig,
    ) -> Result<RedelagationsResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/redelegations");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request(&path, &query).await
    }

    /// Returns the unbonding delegations of given address.
    async fn get_delegator_unbonding_delegations(
        &self,
        delegator_addr: &str,
        pagination_config: &PaginationConfig,
    ) -> Result<UnbondingDelegationResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/unbonding_delegations");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request(&path, &query).await
    }

    /// Returns all the validators by given delegator address.
    async fn get_delegator_validators(
        &self,
        delegator_addr: &str,
        pagination_config: &PaginationConfig,
    ) -> Result<ValidatorsResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/validators");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request(&path, &query).await
    }

    /// Returns validator information by given delegator validator pair.
    async fn get_validator_delegator_pair_info(
        &self,
        delegator_addr: &str,
        validator_addr: &str,
    ) -> Result<ValidatorResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/validators/{validator_addr}");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the signing info by given cons address.
    async fn get_signing_info(&self, cons_addr: &str) -> Result<SigningInfoResp, String> {
        let path = format!("/cosmos/slashing/v1beta1/signing_infos/{cons_addr}");

        self.rest_api_request(&path, &[]).await
    }

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
    async fn get_tallying_params(&self) -> Result<TallyParams, String> {
        self.rest_api_request::<TallyingParamsResp>("/cosmos/gov/v1beta1/params/tallying", &[])
            .await
            .and_then(|res| Ok(res.tally_params))
    }
}

#[derive(Deserialize, Debug)]
pub struct TallyingParamsResp {
    /// Tally parameters.
    tally_params: TallyParams,
}

#[derive(Deserialize, Debug)]
pub struct TallyParams {
    /// Quorum. Eg: `"0.400000000000000000"`
    quorum: String,
    /// Threshold. Eg: `"0.500000000000000000"`
    threshold: String,
    /// Veto threshold. Eg: `"0.334000000000000000"`
    veto_threshold: String,
}

#[derive(Deserialize, Debug)]
pub struct DepositParamsResp {
    /// Deposit parameters.
    deposit_params: DepositParams,
}

#[derive(Deserialize, Debug)]
pub struct DepositParams {
    /// Array of denoms and amounts.
    min_deposit: Vec<DenomAmount>,
    /// Maximum deposit period. Eg: `"0s"`
    max_deposit_period: String,
}

#[derive(Deserialize, Debug)]
pub struct VotingParamsResp {
    /// Voting parameters.
    voting_params: VotingParams,
}

#[derive(Deserialize, Debug)]
pub struct VotingParams {
    /// Voting period. Eg: `"1209600s"`
    voting_period: String,
}

#[derive(Deserialize, Debug)]
pub struct DelegatorRewardsResp {
    /// Array of rewards.
    pub rewards: Vec<DelegatorReward>,
    /// Array of amounts and denoms.
    pub total: Vec<DenomAmount>,
}

#[derive(Deserialize, Debug)]
pub struct DelegatorReward {
    /// Validator address. Eg: `"cosmosvaloper1c4k24jzduc365kywrsvf5ujz4ya6mwympnc4en"`
    pub validator_address: String,
    /// Array of amounts and denoms.
    pub reward: Vec<DenomAmount>,
}

#[derive(Deserialize, Debug)]
pub struct WithdrawAddressResp {
    /// Delegator withdraw address. Eg: `"cosmos1a3yjj7d3qnx4spgvjcwjq9cw9snrrrhu3rw8nv"`
    pub withdraw_address: String,
}

#[derive(Deserialize, Debug)]
pub struct SigningInfoResp {
    /// Validator signing info.
    pub val_signing_info: SlashingSigningInfoItem,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorResp {
    /// Validator.
    pub validator: ValidatorListValidator,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorsResp {
    /// Array of validators.
    pub validators: Vec<ValidatorListValidator>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct UnbondingDelegationResp {
    /// Array of unbonding delegation responses.
    pub unbonding_responses: Vec<UnbondingDelegationResponse>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct UnbondingDelegationResponse {
    /// Delegator address. Eg: `cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6`
    pub delegator_address: String,
    /// Validator address. Eg: `cosmosvaloper156gqf9837u7d4c4678yt3rl4ls9c5vuursrrzf`
    pub validator_address: String,
    // Array of unbonding delegation entries.
    pub entries: Vec<UnbondingDelegationEntry>,
}

#[derive(Deserialize, Debug)]
pub struct UnbondingDelegationEntry {
    /// Unbonding entry creation height. Eg: `"524000"`
    pub creation_height: String,
    /// Unbonding entry competion time. Eg: `"2022-11-06T00:14:50.583Z"`
    pub completion_time: String,
    /// Unbonding entry inital balance. Eg: `""`
    pub initial_balance: String,
    /// Unbonding entry balance. Eg: `""`
    pub balance: String,
}

#[derive(Deserialize, Debug)]
pub struct RedelagationsResp {
    /// Array of redelegation responses.
    pub redelegation_responses: Vec<RedelegationResponse>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct RedelegationResponse {
    /// Delegation.
    pub redelegation: Redelegation,
    /// Amount and denom.
    pub entries: Vec<RedelegationEntry>,
}

#[derive(Deserialize, Debug)]
pub struct Redelegation {
    /// Delegator address. Eg: `"cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6"`
    pub delegator_address: String,
    /// Validator source address. Eg: `""`
    pub validator_src_address: String,
    /// Validator destination address. Eg: `""`
    pub validator_dst_address: String,
    /// Array of redelegation entries.
    pub entries: Vec<RedelegationEntry>,
}

#[derive(Deserialize, Debug)]
pub struct RedelegationResponseEntry {
    /// Redelegation entry.
    pub redelegation_entry: RedelegationEntry,
    /// Balance. Eg: `""`
    pub balance: String,
}

#[derive(Deserialize, Debug)]
pub struct RedelegationEntry {
    /// Redelagation creation height. Eg: `"524000"`
    pub creation_height: String,
    /// Redelagation competion time. Eg: `"2022-11-06T00:14:50.583Z"`
    pub completion_time: String,
    /// Redelagation inital balance. Eg: `""`
    pub initial_balance: String,
    /// Redelagation shares destination. Eg: `""`
    pub shares_dst: String,
}

#[derive(Deserialize, Debug)]
pub struct DelagationsResp {
    /// Array of delegation responses.
    pub delegation_responses: Vec<DelegationResponse>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct DelegationResponse {
    /// Delegation.
    pub delegation: Delegation,
    /// Amount and denom.
    pub balance: DenomAmount,
}

#[derive(Deserialize, Debug)]
pub struct Delegation {
    /// Delegator address. Eg: `"cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6"`
    pub delegator_address: String,
    /// Validator address. Eg: `"cosmosvaloper156gqf9837u7d4c4678yt3rl4ls9c5vuursrrzf"`
    pub validator_address: String,
    /// Delegation shares. Eg: `"1899999.000000000000000000"`
    pub shares: String,
}

#[derive(Deserialize, Debug)]
pub struct ProposalVoteByVoterResp {
    /// Proposal vote.
    pub vote: ProposalVote,
}

#[derive(Deserialize, Debug)]
pub struct ProposalVotesResp {
    /// Array of proposal votes.
    pub votes: Vec<ProposalVote>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct ProposalVote {
    /// Proposal ID. Eg: `"34"`
    pub proposal_id: String,
    /// Proposal voter. Eg: `""`
    pub voter: String,
    /// Proposal vote option. Eg: `"VOTE_OPTION_UNSPECIFIED"`
    pub option: String,
    /// Array of proposal options.
    pub options: Vec<ProposalOption>,
}

#[derive(Deserialize, Debug)]
pub struct ProposalOption {
    /// Proposal vote option. Eg: `"VOTE_OPTION_UNSPECIFIED"`
    pub option: String,
    /// Proposal vote option weight. Eg: `""`
    pub weight: String,
}

#[derive(Deserialize, Debug)]
pub struct ProposalTallyResp {
    /// Proposal tally.
    pub tally: ProposalFinalTallyResult,
}

#[derive(Deserialize, Debug)]
pub struct ProposalDepositByDepositorResp {
    /// Proposal deposit.
    pub deposit: ProposalDeposit,
}

#[derive(Deserialize, Debug)]
pub struct ProposalDepositsResp {
    /// Proposal deposits.
    pub deposits: Vec<ProposalDeposit>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct ProposalDeposit {
    /// Proposal ID. Eg: `"35"`
    pub proposal_id: String,
    /// Proposal depositor. Eg: `""`
    pub depositor: String,
    /// Array of amounts and denoms deposited.
    pub amount: DenomAmount,
}

#[derive(Deserialize, Debug)]
pub struct ProposalsDetailsResp {
    /// Proposal details.
    pub proposal: Proposal,
}

#[derive(Deserialize, Debug)]
pub struct ProposalsResp {
    /// Array of proposals.
    pub proposals: Vec<Proposal>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct Proposal {
    /// Proposal ID. Eg: `"79"`
    pub proposal_id: String,
    /// Proposal content.
    pub content: ProposalContent,
    /// Proposal status. Eg: `"PROPOSAL_STATUS_VOTING_PERIOD"`
    pub status: String,
    /// Proposal final tally result.
    pub final_tally_result: ProposalFinalTallyResult,
    /// Proposal submit time. Eg: `"2022-10-24T19:45:39.969555358Z"`
    pub submit_time: String,
    /// Proposal deposit deadline time. Eg: `"2022-11-07T19:45:39.969555358Z"`
    pub deposit_end_time: String,
    /// Proposal total deposit. Array of amounts and denoms.
    pub total_deposit: Vec<DenomAmount>,
    /// Proposal voting start time. Eg: `"2022-10-24T19:45:39.969555358Z"`
    pub voting_start_time: String,
    /// Proposal voting start time. Eg: `"2022-11-07T19:45:39.969555358Z"`
    pub voting_end_time: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "@type")]
pub enum ProposalContent {
    #[serde(rename = "/cosmos.distribution.v1beta1.CommunityPoolSpendProposal")]
    CommunityPoolSpendProposal {
        /// Community pool spend proposal title. Eg: `"Adan: non-profit fighting for sound crypto regulation"`
        title: String,
        /// Community pool spend proposal description. Eg: `"# Adan: non-profit fighting for sound crypto regulation\n\n## Summary\n\n- Adan is a non-profit organization representing the crypto-asset industry in Europe\n- Since 2020, Adan has worked relentlessly to fight disproportional and overreaching regulations which threaten the crypto industry's ability to innovate\n- We seek 8,000 ATOMS in funding to hire a European Affairs Officer as we ramp up operations in Brussels\n\n## About Adan\n\nAdan is a non-profit trade organization representing the crypto-asset industry. Our members are crypto-native companies and firms that provide industry-specific expertise (legal, consulting, marketing, etc.)\n\nFounded in France in 2020, Adan has over [195 members 6](https://adan.eu/en/association/members) in France and Europe across several industry verticals, including DeFi, payments, market making, custody, data analysis, and security, and is the largest crypto trade organization in Europe.\n\nAdan interacts with all stakeholders with interest in the crypto ecosystem: national and European Members of Parliament, the European Commission, the European Council, the European Parliament, as well as the European Central Bank, and the regulatory bodies and legislators of several European countries.\n\nOur unique positioning allows us to rally decentralized DeFi actors and more traditional listed companies and corporations.\n\n- [Adan Website](https://adan.eu)\n- [Adan on Twitter](https://twitter.com/adan_asso)\n- [Adan on LinkedIn](https://www.linkedin.com/company/adaneu/)\n\n### Mission\n\n- Educate about crypto and help create better narratives around this technology and industry\n- Foster an environment favorable to the growth of the industry\n- Accompany the implementation of French and European regulatory frameworks adapted to the specificities of the sector\n- Build constructive relations between the industry and public institutions\n\n### Team\n\n- Faustine Fleuret: President \u0026 CEO | [Twitter 8](https://twitter.com/faufleuret) / [Linkedin](https://www.linkedin.com/in/faustine-fleuret-640b67a4/)\n- Mélodie Ambroise: Head of Strategy \u0026 Institutional Relations | [Twitter 3](https://twitter.com/mambroise23) / [Linkedin](https://www.linkedin.com/in/m%C3%A9lodie-ambroise/)\n- Jules Dubourg: Administrative \u0026 Financial Manager | [Twitter 1](https://twitter.com/Jules_Dubourg) / [Linkedin 1](https://www.linkedin.com/in/jules-dubourg/)\n- Hugo Bordet: Regulatory Affairs Manager | [Twitter](https://twitter.com/HugoBordet1) / [Linkedin 1](https://www.linkedin.com/in/hugo-bordet-598241152/)\n- Dorian Ravaute: Fiscal Affairs Officer |  [Linkedin 1](https://www.linkedin.com/in/dorianravaute/)\n\n### Funding\n\nAdan is a membership organization and is funded primarily through annual fees.\n\nBeing aware that our missions concern not only the Cosmos community but the whole crypto ecosystem, we will ask for grants from other blockchains. Thus the costs will be shared between different communities.\n\nFinally, we will ask the Cosmos community once only. Our growth plan and the opening of the European market will allow us to be financially self-sufficient through membership fees - which today represent over 80% of our annual income.\n\n### Governance\n\nMembers define the yearly objectives during the annual General Assembly. They also vote on the budget, ratify the association's actions, and elect a Board of Directors responsible for representing them and controlling the implementation of these missions.\n\nSee our website for more information about Adan's governance and [Board of Directors 3](https://adan.eu/en/association/governance).\n\nAdan is a non-profit organization registered in France (Association loi de 1901).\n\n### Works \u0026 Publications\n\nAdan interacts with all stakeholders with an interest in the crypto ecosystem: national and European Members of Parliament, the European Commission, the European Council, the European Parliament, as well as the European Central Bank, and the regulatory bodies and legislators of several European countries.\n\nAdan is also involved in discussions with international bodies such as FATF, IOSO, BIS etc.\n\nIn just two short years, Adan has produced an astounding amount of writing and [publications](https://adan.eu/en/publications) to support its mission:\n\n- [A crypto-euro issued by an American giant, or how Europe is delegating its monetary sovereignty](https://adan.eu/en/tribune-les-echos-crypto-euro-en)\n- [EU framework for crypto-asset markets: the French Presidency ends with political deals on MiCA and TFR](https://adan.eu/en/press-release-policy-agreements-mica)\n- [Adan's Response to IOSCO's Retail Market Conduct Report](https://adan.eu/en/consultation/en/response-report-iosco-retail-market)\n- [Adoption of TFR in the European Parliament: the fight against financial crime should not be a fight against crypto-assets](https://adan.eu/en/press/tfr-travel-rule-vote-european-parliament-europeen-econ-libe)\n- [MiCA vote in the European Parliament: A step forward or backward for the crypto sector?](https://adan.eu/en/press-release/european-parliament-mica-compromise-crypto)\n- [Adan responds to the EBA consultation on its draft guidelines for remote onboarding customer solutions 1](https://adan.eu/en/consultation/response-guidelines-eba-onboarding-solutions)\n- [Ban of Proof-of-Work protocols: wrong answer to real global environmental challenges](https://adan.eu/en/position/pow-bitcoin-ban-eu-inappropriate-answer)\n- [Adan's position on FATF's updated guidance for a risk-based approach 1](https://adan.eu/en/position/fatf-updated-guidance-vasp)\n\n## Proposal details\n\nThe crypto industry is evolving rapidly, challenges are multiplying, and public authorities worldwide are increasingly seeking to regulate crypto innovation.\n\nTo this end, we will continue to fight the battles initiated at the French level but must accelerate on the European and international scene according to 4 priorities:\n\n- Monitor upcoming legislation and regulations, such as MiCA, TFR, and the remainder of the AML package.\n- Contribute to elaborating regulatory frameworks in preparation surrounding topics like DeFi, NFTs, the environmental impact of crypto, etc.\n- Establish strong positions on European crypto companies' issues (e.g., access to banking and insurance) and convey them to policymakers.\n- Sensibly educate on regulatory proposals which fail to capture the unique properties of crypto-economic models and risk hindering innovation (e.g., regulating DAOs as legal persons in the traditional sense).\n\nTo overcome these challenges, our team must engage in the following activities:\n\n- Carry out diligent monitoring of the legislative and regulatory agenda\n- Think up pragmatic solutions adapted to the sector within our working groups\n- Dialogue with decision-makers in European institutions (European Commission, Council of the European Union, European Parliament), European authorities (European Central Bank, European Banking Authority, etc.), and international bodies (FATF, IOSCO, BIS, etc.)\n- Create synergies with other market players with shared interests\n\nGiven the size and importance of the mission, which is constantly expanding, Adan must strengthen its team. Thus we request funding from the Cosmos Hub community pool to recruit a European Affairs Officer based in Brussels, allowing us to further increase our ties with transnational bodies.\n\n## Deliverables\n\nWe believe transparency around community funding is important for building trust and establishing a reputation. This is why we pledge to publish progress reports in 6 and 12 months such that the Cosmos Community better understands how funds are spent and may evaluate the return on its investment.\n\nThis report will be delivered to you via this forum. It will consist of different sections, such as :\n\n- actions carried out in the last months;\n- missions to be carried out in the coming months;\n- distribution of the remaining allocation.\n\n## Funding Amount\n\nTotal amount requested: 8,000 ATOM\n\nThis corresponds to roughly 100,000 EUR and covers gross salary, recruitment costs, and travel expenses for half a year.\n\n## Receipient\n\ncosmos1kdff80vxuj0zmmjauymzjs40hsfkuya79s8tm0\n\n## IPFS\n\nQmR1q2i48EJqaZSXxgggE8VaPKsZFtozVBPHKejMpJAYXx\n\n## Governance Discussion\n\nhttps://forum.cosmos.network/t/proposal-draft-adan-non-profit-fighting-for-sound-crypto-regulation/7416\n\n## Governance votes\n\nThe following items summarize the voting options and what it means for this proposal:\n\n- YES - You agree that persuading regulators to adopt sound and proportional regulation is important and thus wish to fund Adan's work\n- NO - You don't agree that persuading regulators is important and thus do not agree to fund Adan's work\n- NO WITH VETO - A 'NoWithVeto' vote indicates a proposal either (1) is deemed to be spam, i.e., irrelevant to Cosmos Hub, (2) disproportionately infringes on minority interests, or (3) violates or encourages violation of the rules of engagement as currently set out by Cosmos Hub governance. If the number of 'NoWithVeto' votes is greater than a third of the total votes, the proposal is rejected, and the deposits are burned.\n- ABSTAIN - You wish to contribute to the quorum but formally decline to vote either for or against the proposal."`
        description: String,
        /// Community pool spend proposal recipient address. Eg: `"cosmos1kdff80vxuj0zmmjauymzjs40hsfkuya79s8tm0"`
        recipient: String,
        /// Community pool spend proposal amount. Array of amounts and denoms.
        amount: Vec<DenomAmount>,
    },
    #[serde(rename = "/cosmos.gov.v1beta1.TextProposal")]
    TextProposal {
        /// Text proposal title. Eg: `"Risk and financial analysis of ATOM2.0"`
        title: String,
        /// Text proposal description. `"In depth financial analysis of ATOM2.0:\nhttps://pastebin.com/fVQ81d7H\n\nIn depth risk analysis of ATOM2.0:\nhttps://cryptpad.fr/pad/#/2/pad/view/v3QYkKeqenjgK+yPi8bDmuYv4cOBalDaei4sLta6RTg/\nhttps://pastebin.com/bgEqdKct      - backup link\n\n\nWhile many only make claims of faith, these papers make claim of empirical liablity and risk.\n\nGroups to discuss the various proposals:\nhttps://t.me/AtomPrice\nhttps://t.me/atomgov \nhttps://t.me/+uNNyjiYO38lhZDYx\n\nOpen source community lab with the goal of finding an alternative to dilution:\nhttps://forum.cosmos.network/t/atom-zero-a-open-source-non-dilutive-communitylab-for-atom2-0/7860"`
        description: String,
    },
    #[serde(rename = "/cosmos.params.v1beta1.ParameterChangeProposal")]
    ParameterChangeProposal {
        /// Parameter change proposal title. Eg: `"Adjust Blocks Per Year to 4.36M"`
        title: String,
        /// Parameter change proposal description. Eg: `"While the current inflation rate is set at 7%, the effective inflation rate is more like ~6.29%. This is because blocks have slowed down somewhat from ~6.5s to ~7.24s per block, and thus the current blocks per year value of 4855015 is too high. Here we propose to adjust the blocks per year value from 4855015 to 4360000 to bring it in line with current block times, which should realign the effective inflation rate. More details were drafted on Github (https://github.com/cosmos/governance/tree/master/proposals/2020-10-blocks-per-year) and are available on IPFS (https://ipfs.io/ipfs/QmTkzDwWqPbnAh5YiV5VwcTLnGdwSNsNTn2aDxdXBFca7D/example#/ipfs/QmTZ3R4W2odBsx6hpt7awfRTfZA67x5eQaoL6ctxBr6NyN)"`
        description: String,
        /// Array of changes wanted.
        changes: Vec<ParameterChangeProposalChange>,
    },
    #[serde(rename = "/cosmos.upgrade.v1beta1.SoftwareUpgradeProposal")]
    SoftwareUpgradeProposal {
        /// Software upgrade proposal title. Eg: `"Signal Proposal to Adopt the Liquidity Module onto the Cosmos Hub"`
        title: String,
        /// Software upgrade propsal description. Eg: `"Summary: Tendermint (https://tendermint.com) and B-Harvest (https://bharvest.io) have joined forces to produce and develop a Liquidity Module (https://github.com/tendermint/liquidity). This signal proposal is a Request For Comment to the ATOM community regarding the addition of this Liquidity Module into the Cosmos Hub source code.\nBy voting YES to this proposal, you will signal that you approve of having a DEX based on this specific Liquidity Module deployed on the Cosmos Hub.\nDetail of the proposal can be found at IPFS link below.\n\nCurrent Development : https://github.com/tendermint/liquidity/tree/develop\nIPFS : https://ipfs.io/ipfs/QmZpgkYLoCBnXM1S7TEdQunMmur9bAw5VTNgFQCyrqgKDd"`
        description: String,
        /// Software upgrade proposal plan.
        plan: SoftwareUpgradeProposalPlan,
    },
    #[serde(rename = "/ibc.core.client.v1.ClientUpdateProposal")]
    ClientUpdateProposal {
        /// Client update proposal title. Eg: `"Update expired client between Cosmoshub and Bostrom"`
        title: String,
        /// Client update proposal description. Eg: `"This proposal will update the expired client on channel-240 between cosmoshub-4 and the bostrom networks. In turn, this will let users transfer  from bostrom, and to transfer  from cosmoshub back to bostrom.\\n\\nBy voting **YES**, the Cosmoshub stakers, voice their support to unfreeze IBC channel-240 between Cosmoshub and Bostrom.\\n\\nBy voting **NO**, the Cosmoshub stakers voice their dissent to unfreeze IBC channel-240 between Cosmoshub and Bostrom network.\\n\\n**Details:**\\n\\nMost IBC connections between Bostrom and other Cosmos chains have been relayed, to a large extent, only by the Bro_n_Bro validator.\\n\\nOriginally, channel-240 was created with a very short trusting period of 2 days. Alas, the lack of monitoring from our side caused the expiration of client 07-tendermint-497, which in turn, led to the impossibility to transfer tokens using channel-240. Currently, there are around 710 ATOM stuck on the bostrom chain, belonging to about 20 different accounts.\\n\\nAs this might be the first case, when a channel renewal on cosmoshub-4, happens via a governance proposal, we have set up prior testing to ensure that everything will work smoothly. We also modified test-suite https://github.com/bro-n-bro/ibc-testbed (thanks to the Lum devs for the awesome repo), so everyone could simulate the client renewal using governance with this test suite.\\n\\nIn the case that this proposal goes through, client 07-tendermint-497 state will be substituted by the state of client 07-tendermint-643.\\nAlso if passed - channels 240-5 (cosmoshub-4 - bostrom) would be used, only, to recover the stuck funds. New channels would be created with a longer trusting period to ensure further stability.\\n\\nWe will be happy to answer any questions at our [Telegram community group](https://t.me/bro_n_bro_community) or on our [Discord](https://discord.com/channels/868962876721860638/870738846772514826)."`
        description: String,
        /// Client update proposal subject client ID. Eg: `"07-tendermint-497"`
        subject_client_id: String,
        /// Client update proposal substitue client ID. Eg: `"07-tendermint-643"`
        substitute_client_id: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct SoftwareUpgradeProposalPlan {
    /// Software upgrade proposal plan name. Eg: `"Signal Proposal to Adopt the Liquidity Module onto the Cosmos Hub"`
    pub name: String,
    /// Software upgrade proposal plan time. Eg: `"9999-12-31T00:00:00Z"`
    pub time: String,
    /// Software upgrade proposal plan height. Eg: `"0"`
    pub height: String,
    /// Software upgrade proposal plan information. Eg: `"This is information about software upgrade."`
    pub info: String,
    // Software upgrade proposal plan upgraded client state. Can be `None`.
    // TODO! We don't know what it can be.
    // upgraded_client_state: Option<>
}

#[derive(Deserialize, Debug)]
pub struct ParameterChangeProposalChange {
    /// Subspace. Eg: `"mint"`
    pub subspace: String,
    /// Key. Eg: `"BlocksPerYear"`
    pub key: String,
    /// Value. Inside quotes. Eg: `"\"4360000\""`
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct ProposalFinalTallyResult {
    /// Number of `yes` votes. Eg: `"50"`
    pub yes: String,
    /// Number of `abstain` votes. Eg: `"35"`
    pub abstain: String,
    /// Number of `no` votes. Eg: `"12"`
    pub no: String,
    /// Number of `no with veto` votes.  Eg: `"7"`
    pub no_with_veto: String,
}

#[derive(Deserialize, Debug)]
pub struct InflationParams {
    /// Mint denom. Eg: `"aevmos"`
    pub mint_denom: String,
    /// Exponential calculation.
    pub exponential_calculation: InflationParamsExponentialCalculation,
    /// Inflation distribution.
    pub inflation_distribution: InflationParamsInflationDistribution,
    /// Enable inflation. Eg: `true`
    pub enable_inflation: bool,
}

#[derive(Deserialize, Debug)]
pub struct InflationParamsInflationDistribution {
    /// Staking rewards inflation. Eg: `"0.533333334000000000"`
    pub staking_rewards: String,
    /// Usage incentives inflation. Eg: `"0.333333333000000000"`
    pub usage_incentives: String,
    /// Community pool inflation. Eg: `"0.133333333000000000"`
    pub community_pool: String,
}

#[derive(Deserialize, Debug)]
pub struct InflationParamsExponentialCalculation {
    /// Unknown. Eg: `"300000000.000000000000000000"`
    pub a: String,
    /// Unknown. Eg: `"0.500000000000000000"`
    pub r: String,
    /// Unknown. Eg: `"9375000.000000000000000000"`
    pub c: String,
    /// Bonding target. Eg: `"0.660000000000000000"`
    pub bonding_target: String,
    /// Maximum variance. Eg: `"0.000000000000000000"`
    pub max_variance: String,
}

#[derive(Deserialize, Debug)]
pub struct MintParams {
    /// Mint denom. Eg: `"uatom"`
    pub mint_denom: String,
    /// Mint inflation rate change. Eg: ` "1.000000000000000000"`
    pub inflation_rate_change: String,
    /// Maximum mint inflation. Eg: `"0.200000000000000000"`
    pub inflation_max: String,
    /// Minimum mint inflation. Eg: `"0.070000000000000000"`
    pub inflation_min: String,
    /// Goal bonded. Eg: `"0.670000000000000000"`
    pub goal_bonded: String,
    /// Blocks per year. Eg: `"4360000"`
    pub blocks_per_year: String,
}

#[derive(Deserialize, Debug)]
pub struct SlashingParams {
    /// Slashing, signed blocks window. Eg: `"10000"`
    pub signed_blocks_window: String,
    /// Slashing, minimum signed per window. Eg: `"0.050000000000000000"`
    pub min_signed_per_window: String,
    /// Slashing, downtime jail duration. Eg: `"600s"`
    pub downtime_jail_duration: String,
    /// Slash fraction double sign. Eg: `"0.050000000000000000"`
    pub slash_fraction_double_sign: String,
    /// Slash fraction downtime. Eg: `"0.000100000000000000"`
    pub slash_fraction_downtime: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListResp {
    /// Array of validators.
    pub validators: ValidatorListValidator,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidator {
    /// Operator address. Eg: `"evmosvaloper1qq95x6dhrdnrfunlth5uh24tkrfphzl9crd3xr"`
    pub operator_address: String,
    /// Consensus public key.
    pub consensus_pubkey: PublicKey,
    /// Jailed state. Eg: `false`
    pub jailed: bool,
    /// Status. Eg: `"BOND_STATUS_BONDED"`
    pub status: String,
    /// Tokens. Eg: `"145722654634775400576772"`
    pub tokens: String,
    /// Delegator shares. Eg: `"146454922655204548581706.446790192014497216"`
    pub delegator_shares: String,
    /// Description.
    pub description: ValidatorListValidatorDescription,
    /// Unbonding height. Eg: `"2580496"`
    pub unbonding_height: String,
    /// Unbonding time. Eg: `"2022-08-21T03:48:38.952541966Z"`
    pub unbonding_time: String,
    /// Validator commission.
    pub commission: ValidatorListValidatorCommission,
    /// Minimum self delegation. Eg: `"1"`
    pub min_self_delegation: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidatorCommission {
    /// Validator commission rates.
    pub commission_rates: ValidatorListValidatorCommissionRates,
    /// Validator commission update time. Eg: `"2022-03-02T19:00:00Z"`
    pub update_time: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidatorCommissionRates {
    /// Validator commission rate. Eg: `"0.050000000000000000"`
    pub rate: String,
    /// Validator maximum commission rate. Eg: `"0.200000000000000000"`
    pub max_rate: String,
    /// Validator maximum commission change rate. Eg: `"0.010000000000000000"`
    pub max_change_rate: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidatorDescription {
    /// Validator moniker. Eg: `"heisenbug"`
    pub moniker: String,
    /// Validator identity. Eg: `"367960C067E253A4"`
    pub identity: String,
    /// Validator website. Eg: `"https://heisenbug.one"`
    pub website: String,
    /// Validator security contact. Eg: `"@heisenbug_evmos"`
    pub security_contact: String,
    /// Validator details. Eg: `"reliable \u0026\u0026 secure staking"`
    pub details: String,
}

#[derive(Deserialize, Debug)]
pub struct ParamsResp<T> {
    /// The parameters.
    pub params: T,
}

#[derive(Deserialize, Debug)]
pub struct StakingParams {
    /// Unbonding time. Eg: `"1814400s"`
    pub unbonding_time: String,
    /// Maximum number of validators. Eg: `175`
    pub max_validators: usize,
    /// Maximum number of entries. Eg: `7`
    pub max_entries: usize,
    /// Historical number of entries. Eg: `10000`
    pub historical_entries: usize,
    /// Bonding denom. Eg: `"uatom"`
    pub bond_denom: String,
}

#[derive(Deserialize, Debug)]
pub struct MintingInflationResp {
    /// Minting inflation rate. Eg: `"0.131020685388983473"`
    pub inflation: String,
}

#[derive(Deserialize, Debug)]
pub struct MintingInflationRateResp {
    /// Minting inflation rate. Eg: `"91.087708112747866100"`
    pub inflation_rate: String,
}

#[derive(Deserialize, Debug)]
pub struct StakingPoolResp {
    /// Staking pool information.
    pub pool: StakingPool,
}

#[derive(Deserialize, Debug)]
pub struct StakingPool {
    /// Tokens not bonded. Eg: `"15241580330282"`
    pub not_bonded_tokens: String,
    /// Tokens bonded. Eg: `"203496656637783"`
    pub bonded_tokens: String,
}

#[derive(Deserialize, Debug)]
pub struct SupplyByDenomResp {
    /// Amount and denom.
    pub amount: DenomAmount,
}

#[derive(Deserialize, Debug)]
pub struct SupplyOfAllTokensResp {
    /// Array of amounts and denoms.
    pub supply: Vec<DenomAmount>,
    /// Paginations
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorCommisionResp {
    /// Validator commission.
    pub commission: ValidatorCommision,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorCommision {
    /// Array of amounts and demons.
    pub commission: Vec<DenomAmount>,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorRewardsResp {
    /// Validator rewards.
    pub rewards: ValidatorCommision,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorRewards {
    /// Array of amounts and denoms.
    pub rewards: Vec<DenomAmount>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RPCResponse<T> {
    Success(RPCSuccessResponse<T>),
    Error(RPCErrorResponse),
}

#[derive(Deserialize, Debug)]
pub struct TxsResp {
    pub txs: Vec<TxsTransaction>,
    pub tx_responses: Vec<TxResponse>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct TxResponse {
    /// Block height. Eg: `"12713829"`
    pub height: String,
    /// HEX encoded transaction hash. Eg: `"D29DEB0948ADC9B14A1758ED164A46407AF33EA2950404DB4AFFF68164B01C58"`
    pub txhash: String,
    /// Transaction codespace. Eg: `""`
    pub codespace: String,
    /// Code. Eg: `0`
    pub code: usize,
    /// HEX encoded data. Eg: `"0A1E0A1C2F636F736D6F732E62616E6B2E763162657461312E4D736753656E64"`
    pub data: String,
    /// JSON encoded raw log. Eg: `"[{\"events\":[{\"type\":\"coin_received\",\"attributes\":[{\"key\":\"receiver\",\"value\":\"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef\"},{\"key\":\"amount\",\"value\":\"450000uatom\"}]},{\"type\":\"coin_spent\",\"attributes\":[{\"key\":\"spender\",\"value\":\"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf\"},{\"key\":\"amount\",\"value\":\"450000uatom\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"/cosmos.bank.v1beta1.MsgSend\"},{\"key\":\"sender\",\"value\":\"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf\"},{\"key\":\"module\",\"value\":\"bank\"}]},{\"type\":\"transfer\",\"attributes\":[{\"key\":\"recipient\",\"value\":\"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef\"},{\"key\":\"sender\",\"value\":\"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf\"},{\"key\":\"amount\",\"value\":\"450000uatom\"}]}]}]"`
    pub raw_log: String,
    /// Logs.
    pub logs: Vec<TxsResponseLog>,
    /// Info. Eg: `""`
    pub info: String,
    // Gas wanted. Eg: `"80000"`
    pub gas_wanted: String,
    /// Gas used. Eg: `"74032"`
    pub gas_used: String,
    // Tx.
    pub tx: TxsResponseTx,
    // Timestamp. Eg: `"2022-07-19T05:26:26Z"`
    pub timestamp: String,
    // Events.
    pub events: Vec<TxsResponseEvent<TransactionEventAttribute>>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "@type")]
pub enum TxsResponseTx {
    #[serde(rename = "/cosmos.tx.v1beta1.Tx")]
    Tx {
        // Tx body.
        body: TxsTransactionBody,
        // Tx auth info.
        auth_info: TxsTransactionAuthInfo,
        /// Array of Base 64 encoded signatures.
        signatures: Vec<String>,
    },
}

#[derive(Deserialize, Debug)]
pub struct TxsResponseLog {
    /// Message index. Eg: `0`
    pub msg_index: usize,
    /// Log. Eg: `""`
    pub log: String,
    /// Events.
    pub events: Vec<TxsResponseEvent<TxsResponseLogEventAttribute>>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum TxsResponseEvent<T> {
    CoinReceived {
        /// Coin received attributes.
        attributes: Vec<T>,
    },
    CoinSpent {
        /// Coin spent attributes.
        attributes: Vec<T>,
    },
    Message {
        /// Message attributes.
        attributes: Vec<T>,
    },
    Transfer {
        /// Transfer attributes.
        attributes: Vec<T>,
    },
    Tx {
        /// Tx attributes.
        attributes: Vec<T>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "key")]
#[serde(rename_all = "lowercase")]
pub enum TxsResponseLogEventAttribute {
    Receiver {
        /// Receiver address. Eg: `"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef"`
        #[serde(rename = "value")]
        receiver_address: String,
    },
    Amount {
        /// Received amount. Eg: `"450000uatom"`
        #[serde(rename = "value")]
        amount: String,
    },
    Spender {
        /// Spender address. Eg: `"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf"`
        #[serde(rename = "value")]
        spender_address: String,
    },
    Action {
        /// Action method. Eg: `"/cosmos.bank.v1beta1.MsgSend"`
        #[serde(rename = "value")]
        action_method: String,
    },
    Sender {
        /// Sender address. Eg: `"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf"`
        #[serde(rename = "value")]
        sender_address: String,
    },
    Module {
        /// Module type. Eg: `"bank"`
        #[serde(rename = "value")]
        module_type: String,
    },
    Recipient {
        /// Recipient address. Eg: `"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef"`
        #[serde(rename = "value")]
        recipient_address: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct TxsTransaction {
    /// Transaction body.
    pub body: TxsTransactionBody,
    /// Transaction auth information.
    pub auth_info: TxsTransactionAuthInfo,
    /// Array of Base 64 encoded transaction signatures.
    pub signatures: Vec<String>,
}
#[derive(Deserialize, Debug)]
pub struct TxsTransactionAuthInfo {
    /// Transaction signer informations.
    pub signer_infos: Vec<TxsTransactionSignerInfo>,
    /// Transaction fee.
    pub fee: TxsTransactionAuthInfoFee,
    /// Transaction gas limit.
    pub gas_limit: String,
    /// Transaction payer. Eg: `""`
    pub payer: String,
    /// Transaction granter. Eg: `""`
    pub granter: String,
}

#[derive(Deserialize, Debug)]
pub struct TxsTransactionAuthInfoFee {
    /// Amount.
    pub amount: Vec<DenomAmount>,
}

#[derive(Deserialize, Debug)]
pub struct TxsTransactionBody {
    /// Transaction messages.
    pub messages: Vec<TxsTransactionMessage>,
    /// Transaction memo. Eg: `"1891420480"`
    pub memo: String,
    /// Transaction timeout height. Eg: `"0"`
    pub timeout_height: String,
    /// Transaction extension options.
    pub extension_options: Vec<u8>,
    /// Non-critical transaction extension options.
    pub extension_optionsnon_critical_extension_options: Vec<u8>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "@type")]
pub enum TxsTransactionMessage {
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
    MsgSend {
        /// The address transaction is from. Eg: `"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf"`
        from_address: String,
        /// The address transaction is to. Eg: `"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef"`
        to_address: String,
        /// Transaction amounts.
        amount: Vec<DenomAmount>,
    },
}

#[derive(Deserialize, Debug)]
pub struct TxsTransactionSignerInfo {
    pub public_key: PublicKey,
    pub mode_info: TxsTransactionModeInfo,
    /// Transaction signer info sequence. Eg: `"1"`
    pub sequence: String,
}

#[derive(Deserialize, Debug)]
pub struct TxsTransactionModeInfo {
    pub single: TxsTransactionModeInfoSingle,
}

#[derive(Deserialize, Debug)]
pub struct TxsTransactionModeInfoSingle {
    /// Mode. Eg: `"SIGN_MODE_LEGACY_AMINO_JSON"`
    pub mode: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "@type")]
pub enum PublicKey {
    #[serde(rename = "/cosmos.crypto.secp256k1.PubKey")]
    Secp256K1 {
        /// Base 64 encoded Secp256K1 public key. Eg: `"Ap9xAyS21AGuRY4W7+Mi3JzbmULJjGATAzVeIxc98t07"`
        key: String,
    },
    #[serde(rename = "/cosmos.crypto.ed25519.PubKey")]
    Ed25519 {
        /// Base 64 encoded Ed25519 public key. Eg: `"zy/GxGwk1Pm3HiG67iani1u+MUieM98ZvSIrXC8mISE="`
        key: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct DenomAmount {
    /// The name of the token. Eg: `"uatom"`
    pub denom: String,
    /// The amount of the token. Eg: `"450000"`
    pub amount: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionResp {
    /// HEX encoded TX hash, without leading `0x`. Eg: `"25EC6BCEA9B4A6835F5A38AB566959187F968C295EE71D015C3D907B25C5C72F"`
    pub hash: String,
    /// The block height TX at. Eg: `"6684890"`
    pub height: String,
    /// Unknown. Eg: `0`
    pub index: usize,
    /// The transaction result.
    pub tx_result: TransactionResult,
}

#[derive(Deserialize, Debug)]
pub struct TransactionResult {
    /// Unknown. Eg: `0`
    pub code: usize,
    /// Base64 encoded transaction data. Eg: `"CrgECh8vZXRoZXJtaW50LmV2bS52MS5Nc2dFdGhlcmV1bVR4EpQECkIweDgxNTRhOGEyYmViYzQyYzNhNmVlYTZjMTAwMDMwMzkwMzhkOTJiZGYxOWNiMmQ4NDBhYzJkN2Q2ZmI3YjBmMzISpwMKKjB4NEY0MWE5ZTJjYTc4YWQ2QjZlRmFiNTJGNjYxQjVmMEEwQzIxMUY3NRJCMHg4YzViZTFlNWViZWM3ZDViZDE0ZjcxNDI3ZDFlODRmM2RkMDMxNGMwZjdiMjI5MWU1YjIwMGFjOGM3YzNiOTI1EkIweDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDcyODYxOWNlZjE0MTEyZjFiNzc3ZTQ2ODAwYTkwNjc3ZDQ5OTI1NDQSQjB4MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDY3ZWM4Nzg0NGZiZDczZWRhNGExMDU5ZjMwMDM5NTg0NTg2ZTA5ZBog//////////////////////////////////////////8g2oGYAypCMHg4MTU0YThhMmJlYmM0MmMzYTZlZWE2YzEwMDAzMDM5MDM4ZDkyYmRmMTljYjJkODQwYWMyZDdkNmZiN2IwZjMyOkIweDBhMjdkZDQyNDBkYzM1MjE1OWYxZTVhMzA3NjM0NDIwZmFjN2I2ZDg5YzYxYWI5NzIyNDI4MjIxZWFmYjg4NGYaIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABKKbqAg=="`
    pub data: String,
    /// JSON encoded transaction log. Eg: `"[{\"events\":[{\"type\":\"coin_received\",\"attributes\":[{\"key\":\"receiver\",\"value\":\"evmos1w2rpnnh3gyf0rdmhu35qp2gxwl2fjf2y4vjkhg\"},{\"key\":\"amount\",\"value\":\"199391000000000aevmos\"}]},{\"type\":\"coin_spent\",\"attributes\":[{\"key\":\"spender\",\"value\":\"evmos17xpfvakm2amg962yls6f84z3kell8c5ljcjw34\"},{\"key\":\"amount\",\"value\":\"199391000000000aevmos\"}]},{\"type\":\"ethereum_tx\",\"attributes\":[{\"key\":\"amount\",\"value\":\"0\"},{\"key\":\"ethereumTxHash\",\"value\":\"0x8154a8a2bebc42c3a6eea6c10003039038d92bdf19cb2d840ac2d7d6fb7b0f32\"},{\"key\":\"txIndex\",\"value\":\"0\"},{\"key\":\"txGasUsed\",\"value\":\"46374\"},{\"key\":\"txHash\",\"value\":\"25EC6BCEA9B4A6835F5A38AB566959187F968C295EE71D015C3D907B25C5C72F\"},{\"key\":\"recipient\",\"value\":\"0x4F41a9e2ca78ad6B6eFab52F661B5f0A0C211F75\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"/ethermint.evm.v1.MsgEthereumTx\"},{\"key\":\"sender\",\"value\":\"evmos17xpfvakm2amg962yls6f84z3kell8c5ljcjw34\"},{\"key\":\"module\",\"value\":\"evm\"},{\"key\":\"sender\",\"value\":\"0x728619cEf14112F1B777E46800a90677d4992544\"},{\"key\":\"txType\",\"value\":\"2\"}]},{\"type\":\"transfer\",\"attributes\":[{\"key\":\"recipient\",\"value\":\"evmos1w2rpnnh3gyf0rdmhu35qp2gxwl2fjf2y4vjkhg\"},{\"key\":\"sender\",\"value\":\"evmos17xpfvakm2amg962yls6f84z3kell8c5ljcjw34\"},{\"key\":\"amount\",\"value\":\"199391000000000aevmos\"}]},{\"type\":\"tx_log\",\"attributes\":[{\"key\":\"txLog\",\"value\":\"{\\\"address\\\":\\\"0x4F41a9e2ca78ad6B6eFab52F661B5f0A0C211F75\\\",\\\"topics\\\":[\\\"0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925\\\",\\\"0x000000000000000000000000728619cef14112f1b777e46800a90677d4992544\\\",\\\"0x000000000000000000000000067ec87844fbd73eda4a1059f30039584586e09d\\\"],\\\"data\\\":\\\"//////////////////////////////////////////8=\\\",\\\"blockNumber\\\":6684890,\\\"transactionHash\\\":\\\"0x8154a8a2bebc42c3a6eea6c10003039038d92bdf19cb2d840ac2d7d6fb7b0f32\\\",\\\"transactionIndex\\\":0,\\\"blockHash\\\":\\\"0x0a27dd4240dc352159f1e5a307634420fac7b6d89c61ab9722428221eafb884f\\\",\\\"logIndex\\\":0}\"}]}]}]"`
    pub log: String,
    /// The transaction information. Eg: `""`
    pub info: String,
    /// Gas wanted. Eg: `"55648"`
    pub gas_wanted: String,
    /// Gas used. Eg: `"46374"`
    pub gas_used: String,
    /// Transaction events.
    pub events: Vec<TransactionEvent>,
    /// Transaction codespace. Eg: `""`
    pub codespace: String,
    // Base 64 encoded transaction. Eg: `"CqMDCu8CCh8vZXRoZXJtaW50LmV2bS52MS5Nc2dFdGhlcmV1bVR4EssCCoQCCh4vZXRoZXJtaW50LmV2bS52MS5EeW5hbWljRmVlVHgS4QEKBDkwMDEQKRoKMTUwMDAwMDAwMCILMjU1MDAwMDAwMDAo4LIDMioweDRGNDFhOWUyY2E3OGFkNkI2ZUZhYjUyRjY2MUI1ZjBBMEMyMTFGNzU6ATBCRAlep7MAAAAAAAAAAAAAAAAGfsh4RPvXPtpKEFnzADlYRYbgnf//////////////////////////////////////////UgEBWiD4q5dJAnhCoLGbgwyqtMO3GuL4kx1WmrtUyDr7hzaeYmIgL/+FNXRbiS+/RyH2p5dwQ0O8OOcFHGxUDg6AP3gPYxQaQjB4ODE1NGE4YTJiZWJjNDJjM2E2ZWVhNmMxMDAwMzAzOTAzOGQ5MmJkZjE5Y2IyZDg0MGFjMmQ3ZDZmYjdiMGYzMvo/LgosL2V0aGVybWludC5ldm0udjEuRXh0ZW5zaW9uT3B0aW9uc0V0aGVyZXVtVHgSIhIgChoKBmFldm1vcxIQMTQxOTAyNDAwMDAwMDAwMBDgsgM="`
    pub tx: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionEvent {
    /// Transaction event type. Eg: `"coin_spent"`
    pub r#type: String,
    /// Transaction event attributes.
    pub attributes: Vec<TransactionEventAttribute>,
}

#[derive(Deserialize, Debug)]
pub struct TransactionEventAttribute {
    /// Base 64 encoded transaction event attribute key. Eg: `"c3BlbmRlcg=="`
    pub key: String,
    /// Base 64 encoded transaction event attribute value. Eg: `"ZXZtb3MxdzJycG5uaDNneWYwcmRtaHUzNXFwMmd4d2wyZmpmMnk0dmpraGc="`
    pub value: String,
    /// Transaction event attribute index. Eg: `true`
    pub index: bool,
}

/// The configuration to be used while making REST API requests.
pub struct PaginationConfig {
    /// It is set to true if results are to be returned in the descending order.
    pub reverse: bool,
    pub offset: u64,
    /// It is the total number of results to be returned in the result page
    pub limit: u64,
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    /// Pagination next key. Might be `None`. Eg: `"FGxWOxzuw4bZozVHta3qYgdKOuRC"`
    pub next_key: Option<String>,
    /// Total. Eg: `"0"`
    pub total: String,
}

#[derive(Deserialize, Debug)]
pub struct SlashingSigningInfo {
    pub info: Vec<SlashingSigningInfoItem>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct SlashingSigningInfoItem {
    /// Validator address. Eg: `"evmosvalcons1qx4hehfny66jfzymzn6d5t38m0ely3cvw6zn06"`
    pub address: String,
    /// The block height slashing is started at. Eg: `"0"`
    pub start_height: String,
    /// Unknown. Eg: `"5888077"`
    pub index_offset: String,
    /// The time jailed until. Eg: `"2022-05-14T04:31:49.705643236Z"`
    pub jailed_until: String,
    /// Tombstoned state. Eg: `false`
    pub tombstoned: bool,
    /// The count of missed blocks. Eg: `"16433"`
    pub missed_blocks_counter: String,
}

#[derive(Deserialize, Debug)]
pub struct RPCSuccessResponse<T> {
    pub jsonrpc: String,
    pub id: isize,
    pub result: T,
}

#[derive(Deserialize, Debug)]
pub struct RPCErrorResponse {
    pub jsonrpc: String,
    pub id: isize,
    pub error: RpcErrorResponseError,
}

#[derive(Deserialize, Debug)]
pub struct RpcErrorResponseError {
    /// The error code.
    pub code: isize,
    /// The message about error type.
    pub message: String,
    /// Description about error.
    pub data: String,
}

#[derive(Deserialize, Debug)]
pub struct BlockResp {
    pub block_id: BlockId,
    pub block: BlockBlock,
}

#[derive(Deserialize, Debug)]
pub struct BlockId {
    /// HEX encoded transaction hash.
    pub hash: String,
    pub parts: BlockIdParts,
}

#[derive(Deserialize, Debug)]
pub struct BlockBlock {
    pub header: BlockHeader,
    pub data: BlockData,
    pub evidence: BlockEvidence,
    pub last_commit: BlockLastCommit,
}

#[derive(Deserialize, Debug)]
pub struct BlockIdParts {
    /// Unknown. Eg: `1`
    pub total: usize,
    /// HEX encoded transaction hash.
    pub hash: String,
}

#[derive(Deserialize, Debug)]
pub struct BlockHeader {
    /// Block header version.
    pub version: BlockHeaderVersion,
    /// The ID of the chain. Eg: `"axelar-dojo-1"`
    pub chain_id: String,
    /// The current block height. Eg: `"4611328"`
    pub height: String,
    /// The current block time. Eg: `"2022-11-03T17:45:14.115240656Z"`
    pub time: String,
    /// Last block ID.
    pub last_block_id: BlockId,
    /// HEX encoded transaction hash.
    pub last_commit_hash: String,
    /// HEX encoded transaction hash.
    pub data_hash: String,
    /// HEX encoded transaction hash.
    pub validators_hash: String,
    /// HEX encoded transaction hash.
    pub next_validators_hash: String,
    /// HEX encoded transaction hash.
    pub consensus_hash: String,
    /// HEX encoded transaction hash.
    pub app_hash: String,
    /// HEX encoded transaction hash.
    pub last_results_hash: String,
    /// HEX encoded transaction hash.
    pub evidence_hash: String,
    /// HEX encoded address.
    pub proposer_address: String,
}

#[derive(Deserialize, Debug)]
pub struct BlockData {
    /// Array of very long Base64 encoded transactions. Eg: `["CoYBCoMBCiUvYXhlbGFyLmF4ZWxhcm5ldC52MWJldGExLkxpbmtSZXF1ZXN0EloKFAfFBMRZ8AeNGGkWVAcX+idm5UutEioweDM1NzkyNTRmNTgwNWQxNjZiNjhhNTg3MzIwNzA0NDQ4MjBmYTRiZjEaCGV0aGVyZXVtIgx3YnRjLXNhdG9zaGkSlQEKUQpGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQPUmMSQ2WoB0eD589u7pruIZt2gbHT2DO3QSIPX0z8WXBIECgIIARiuCBJACgsKBHVheGwSAzY3NRDh8AUiLWF4ZWxhcjFwdTJzd2MwbjB0cmZ0bGRoejU3cHlxa3c2ZDg3aGFobjdnNjk3YxpANmM1rQE1P3hbVtuFoaQEpGpnBnlygbotxEA0qR/rmAwVRB+acJ6idoF1V0Qul5eSCpi1Z0TLLwQEMya4nMdl3g=="]`
    pub txs: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct BlockEvidence {
    // Property below is an unknown array. TODO!
    // evidence: Vec<UNKNOWN>
}
#[derive(Deserialize, Debug)]
pub struct BlockLastCommit {
    /// The block height of the latest commit. Eg: `"4611327"`
    pub height: String,
    /// Unknown. Eg: `0`
    pub round: usize,
    /// Block ID.
    pub block_id: BlockId,
    /// Array of signatures.
    pub signatures: Vec<BlockLastCommitSignatures>,
}

#[derive(Deserialize, Debug)]
pub struct BlockHeaderVersion {
    /// Unknown. Eg: `"11"`
    pub block: String,
}

#[derive(Deserialize, Debug)]
pub struct BlockLastCommitSignatures {
    /// Unknown. Eg: `2`
    pub block_id_flag: usize,
    /// HEX encoded address of a validator.
    pub validator_address: String,
    /// The time of the unix timestamp. Eg: `"2022-11-03T17:45:14.193617481Z"`
    pub timestamp: String,
    /// Base 64 encoded signature. It might be `None`, so unsigned. Eg: `"rum2poquBDmHkGLGxjtjrlNBP5bV52m6ckexmNHdln85WRii4tCaVqAmxAKR+fP+hzoxEDuhOGwQ/xlgMVFrAA=="`
    pub signature: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BlockchainResp {
    /// Last block height. `"12733014"`
    pub last_height: String,
    /// Array of block metas.
    pub block_metas: Vec<BlockMeta>,
}

#[derive(Deserialize, Debug)]
pub struct BlockMeta {
    /// Last block height. `"12733014"`
    pub block_id: BlockId,
    /// Block size. Eg: `"13971"`
    pub block_size: String,
    /// Block header.
    pub header: BlockHeader,
    /// Number of transactions. Eg: `"3"`
    pub num_txs: String,
}
