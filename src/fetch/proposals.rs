use chrono::DateTime;
use cosmrs::tendermint::proposal;
use futures::TryFutureExt;
use prost_wkt_types::Timestamp;
use serde::{Deserialize, Serialize};
use tonic::transport::Endpoint;

use super::others::{DenomAmount, Pagination, PaginationConfig};
use crate::{
    chain::Chain,
    database::ListDbResult,
    fetch::{
        cosmos::{
            base::query::v1beta1::{PageRequest, PageResponse},
            distribution::v1beta1::CommunityPoolSpendProposal,
            gov::v1::MsgExecLegacyContent,
            gov::{v1::Deposit, v1beta1::TextProposal},
            params::v1beta1::ParameterChangeProposal,
            upgrade::v1beta1::SoftwareUpgradeProposal,
        },
        evmos::{
            erc20::v1::{RegisterCoinProposal, RegisterErc20Proposal, ToggleTokenConversionProposal},
            incentives::v1::RegisterIncentiveProposal,
        },
        gravity::v1::IbcMetadataProposal,
        ibc::core::client::v1::ClientUpdateProposal,
        osmosis::poolincentives::v1beta1::UpdatePoolIncentivesProposal,
        quicksilver::interchainstaking::v1::RegisterZoneProposal,
        umee::leverage::v1::MsgGovUpdateRegistry,
    },
    routes::OutRestResponse,
    routes::PaginationData,
    routes::{calc_pages, ProposalStatus},
    routes::{ChainAmountItem, PaginationDirection},
};

use prost::Message;

#[derive(Clone)]
pub struct ProposalInfo(String, String, serde_json::Value);

impl From<prost_wkt_types::Any> for ProposalInfo {
    fn from(content: prost_wkt_types::Any) -> ProposalInfo {
        let (title, description, content) = match content.type_url.as_str() {
            "/cosmos.params.v1beta1.ParameterChangeProposal" => {
                let value = ParameterChangeProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/cosmos.upgrade.v1beta1.SoftwareUpgradeProposal" => {
                let value = SoftwareUpgradeProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/cosmos.distribution.v1beta1.CommunityPoolSpendProposal" => {
                let value = CommunityPoolSpendProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/cosmos.gov.v1beta1.TextProposal" => {
                let value = TextProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/ibc.core.client.v1.ClientUpdateProposal" => {
                let value = ClientUpdateProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/evmos.erc20.v1.RegisterCoinProposal" => {
                let value = RegisterCoinProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/evmos.erc20.v1.ToggleTokenConversionProposal" => {
                let value = ToggleTokenConversionProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/evmos.erc20.v1.RegisterERC20Proposal" => {
                let value = RegisterErc20Proposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/osmosis.poolincentives.v1beta1.UpdatePoolIncentivesProposal" => {
                let value = UpdatePoolIncentivesProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/gravity.v1.IBCMetadataProposal" => {
                let value = IbcMetadataProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/umee.leverage.v1.MsgGovUpdateRegistry" => {
                let value = MsgGovUpdateRegistry::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/evmos.incentives.v1.RegisterIncentiveProposal" => {
                let value = RegisterIncentiveProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/quicksilver.interchainstaking.v1.RegisterZoneProposal" => {
                let value = RegisterZoneProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }

            other => {
                dbg!(other);
                (String::from(""), String::from(""), serde_json::Value::Null)
            }
        };
        ProposalInfo(title, description, content)
    }
}

impl Into<PageRequest> for PaginationData {
    fn into(self) -> PageRequest {
        PageRequest {
            key: self.cursor.unwrap_or_else(|| "".to_string()).as_bytes().to_vec(),
            offset: self.offset.unwrap_or_else(|| 0),
            limit: self.limit.unwrap_or_else(|| 20),
            count_total: true,
            reverse: false,
        }
    }
}

impl From<PageResponse> for PaginationData {
    fn from(value: PageResponse) -> Self {
        let cursor = if !value.next_key.is_empty() {
            Some(String::from_utf8(value.next_key).unwrap())
        } else {
            None
        };

        Self {
            cursor,
            offset: None,
            limit: None,
            direction: Some(PaginationDirection::Next),
        }
    }
}

impl Chain {
    async fn get_proposals_v1(&self, status: &str, config: PaginationData) -> Result<ListDbResult<ProposalItem>, String> {
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, Proposal, QueryProposalsRequest, QueryProposalsResponse};
        let config = config.clone();
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let proposal_request = QueryProposalsRequest {
            proposal_status: status.parse().unwrap(),
            voter: "".to_string(),
            depositor: "".to_string(),
            pagination: Some(PageRequest {
                key: config.cursor.unwrap_or_else(|| "".to_string()).as_bytes().to_vec(),
                offset: 0,
                limit: config.limit.unwrap_or_else(|| 50),
                count_total: false,
                reverse: false,
            }),
        };

        let resp = QueryClient::connect(endpoint.clone())
            .await
            .unwrap()
            .proposals(proposal_request)
            .await
            .map_err(|e| format!("{}", e))?;
        let proposals = resp.into_inner();

        let mut items = Vec::with_capacity(proposals.proposals.len());
        for proposal in proposals.proposals {
            if let Some(content) = proposal.messages.get(0).cloned() {
                let content = if content.type_url == "/cosmos.gov.v1.MsgExecLegacyContent" {
                    let legacy_content = MsgExecLegacyContent::decode(content.value.as_ref()).unwrap();
                    legacy_content.content
                } else {
                    Some(content)
                };
                let ProposalInfo(title, description, content) = content.unwrap().into();
                let Proposal { id, submit_time, status, .. } = proposal;
                let proposal_item = ProposalItem {
                    proposal_id: id,
                    title,
                    description,
                    time: submit_time.map(|t| t.seconds),
                    status,
                    content,
                };

                items.push(proposal_item);
            };
        }

        let limit = items.len() as u64;

        Ok(ListDbResult {
            data: items,
            pagination: PaginationData {
                cursor: proposals.pagination.map(|p| String::from_utf8(p.next_key).unwrap()),
                offset: None,
                limit: Some(limit),
                direction: Some(PaginationDirection::Next),
            },
        })
    }
    async fn get_proposals_v1beta1(&self, status: &str, config: PaginationData) -> Result<ListDbResult<ProposalItem>, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, Proposal, QueryProposalsRequest, QueryProposalsResponse, TextProposal};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let proposal_request = QueryProposalsRequest {
            proposal_status: status.parse().unwrap(),
            voter: "".to_string(),
            depositor: "".to_string(),
            pagination: Some(PageRequest {
                key: config.cursor.unwrap_or_else(|| "".to_string()).as_bytes().to_vec(),
                offset: 0,
                limit: config.limit.unwrap_or_else(|| 50),
                count_total: false,
                reverse: false,
            }),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .proposals(proposal_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let proposals = resp.into_inner();

        let mut items = Vec::with_capacity(proposals.proposals.len());
        for proposal in proposals.proposals {
            let Proposal {
                proposal_id,
                submit_time,
                status,
                ..
            } = proposal;
            if let Some(content) = proposal.content {
                let ProposalInfo(title, description, content) = content.into();
                let proposal_item = ProposalItem {
                    proposal_id,
                    title,
                    description,
                    time: submit_time.map(|t| t.seconds),
                    status,
                    content,
                };

                items.push(proposal_item);
            };
        }

        let limit = items.len() as u64;

        Ok(ListDbResult {
            data: items,
            pagination: PaginationData {
                cursor: proposals.pagination.map(|p| String::from_utf8(p.next_key).unwrap()),
                offset: None,
                limit: Some(limit),
                direction: Some(PaginationDirection::Next),
            },
        })
    }
    /// Returns all the proposals in voting period.
    pub async fn get_proposals_by_status(&self, status: ProposalStatus, config: PaginationData) -> Result<ListDbResult<ProposalItem>, String> {
        let status_id = status.get_id().to_string();
        let items = if dbg!(self.config.sdk_version.minor) >= 46 {
            self.get_proposals_v1(&status_id, config.clone()).await.ok()
        } else {
            None
        };

        let items = if let Some(items) = items {
            items
        } else {
            self.get_proposals_v1beta1(&status_id, config)
                .await
                .map_err(|e| format!("Upstream error: {}", e))?
        };

        Ok(items)
    }

    async fn get_proposal_details_v1(&self, proposal_id: u64) -> Result<InternalProposal, String> {
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryProposalRequest, QueryProposalResponse};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let proposal_request = QueryProposalRequest { proposal_id };

        let client = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .proposal(proposal_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let proposal_resp = client.into_inner();
        let proposal = proposal_resp.proposal.ok_or_else(|| String::from("No proposal content"))?;
        let tally_result = proposal.final_tally_result.map(|t| InternalProposalFinalTallyResult {
            yes_count: t.yes_count,
            abstain_count: t.abstain_count,
            no_count: t.no_count,
            no_with_veto_count: t.no_with_veto_count,
        });
        let messages = proposal.messages.into_iter().map(|m| m.into()).collect();
        let internal_proposal = InternalProposal {
            id: proposal.id,
            messages: messages,
            status: ProposalStatus::from_id(proposal.status),
            final_tally_result: tally_result,
            submit_time: proposal.submit_time,
            deposit_end_time: proposal.deposit_end_time,
            total_deposit: ChainAmountItem::default(),
            voting_start_time: proposal.voting_start_time,
            voting_end_time: proposal.voting_end_time,
            metadata: Some(proposal.metadata),
            title: proposal.title,
            summary: proposal.summary,
            proposer: Some(proposal.proposer),
            expedited: Some(proposal.expedited),
        };

        Ok(internal_proposal)
    }
    async fn get_proposal_details_v1beta1(&self, proposal_id: u64) -> Result<InternalProposal, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, Proposal, QueryProposalRequest,TallyResult};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let proposal_request = QueryProposalRequest { proposal_id };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .proposal(proposal_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let proposal_resp = resp.into_inner();
        let proposal = proposal_resp.proposal.ok_or_else(|| String::from("No proposal content"))?;
    

        let prop_info:Option<ProposalInfo> = proposal.content.map(|c| c.into());
        
        
        let (title,summary) = prop_info.clone().map_or_else(||(String::from(""),String::from("")),|p|(p.0.clone(),p.1.clone()));
        let mut messages = vec![];
        if let Some(p) = prop_info {
            messages.push(p);
        }
        let final_tally_result = proposal.final_tally_result.map(|t|InternalProposalFinalTallyResult{ yes_count: t.yes, abstain_count: t.abstain, no_count: t.no, no_with_veto_count:t.no_with_veto});
        
        let internal_proposal = InternalProposal {
            id: proposal_id,
            messages,
            status: ProposalStatus::from_id(proposal.status),
            final_tally_result,
            submit_time:proposal.submit_time,
            deposit_end_time:proposal.deposit_end_time,
            total_deposit: ChainAmountItem::default(),
            voting_start_time:proposal.voting_start_time,
            voting_end_time:proposal.voting_end_time,
            title,
            summary,
            metadata: None,
            proposer: None,
            expedited: None,
        };

        Ok(internal_proposal)
    }
    /// Returns the details of given proposal.
    pub async fn get_proposal_details(&self, proposal_id: u64) -> Result<InternalProposal, String> {
        let items = if dbg!(self.config.sdk_version.minor) >= 46 {
            self.get_proposal_details_v1(proposal_id).await.ok()
        } else {
            None
        };

        let items = if let Some(items) = items {
            items
        } else {
            self.get_proposal_details_v1beta1(proposal_id)
                .await
                .map_err(|e| format!("Upstream error: {}", e))?
        };

        Ok(items)
    }

    async fn proposal_deposits_v1(&self, proposal_id: u64, config: PaginationData) -> Result<ListDbResult<InternalProposalDeposit>, String> {
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryDepositResponse, QueryDepositsRequest};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let deposits_request = QueryDepositsRequest {
            proposal_id,
            pagination: Some(config.into()),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .deposits(deposits_request)
            .await
            .map_err(|e| format!("{}", e))?;
        let deposits = resp.into_inner();

        let internal_deposits = deposits
            .deposits
            .iter()
            .map(|d| InternalProposalDeposit {
                depositor: d.depositor.clone(),
                //TODO map with amount denom utils
                amount: 0.0,
            })
            .collect();

        Ok(ListDbResult {
            data: internal_deposits,
            pagination: deposits.pagination.map(|p| p.into()).unwrap_or_default(),
        })
    }

    async fn proposal_deposits_v1beta1(&self, proposal_id: u64, config: PaginationData) -> Result<ListDbResult<InternalProposalDeposit>, String> {
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryDepositsRequest, QueryDepositsResponse};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let deposit_request = QueryDepositsRequest {
            proposal_id,
            pagination: Some(config.into()),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .deposits(deposit_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let deposits = resp.into_inner();
        let internal_deposits = deposits
            .deposits
            .iter()
            .map(|d| InternalProposalDeposit {
                depositor: d.depositor.clone(),
                //TODO map with amount denom utils
                amount: 0.0,
            })
            .collect();

        Ok(ListDbResult {
            data: internal_deposits,
            pagination: deposits.pagination.map(|p| p.into()).unwrap_or_default(),
        })
    }
    /// Returns the deposits of given proposal.
    pub async fn get_proposal_deposits(&self, proposal_id: u64, config: PaginationData) -> Result<ListDbResult<InternalProposalDeposit>, String> {
        let items = if dbg!(self.config.sdk_version.minor) >= 46 {
            self.proposal_deposits_v1(proposal_id, config.clone()).await.ok()
        } else {
            None
        };

        let items = if let Some(items) = items {
            items
        } else {
            self.proposal_deposits_v1beta1(proposal_id, config)
                .await
                .map_err(|e| format!("Upstream error: {}", e))?
        };

        Ok(items)
    }

    async fn proposal_deposit_v1(&self, proposal_id: u64, depositor: &str) -> Result<InternalProposalDeposit, String> {
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryDepositRequest, QueryDepositResponse};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let deposit_request = QueryDepositRequest {
            proposal_id,
            depositor: depositor.to_string(),
        };

        let client = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .deposit(deposit_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let deposit = client.into_inner();

        let internal_deposit = InternalProposalDeposit {
            depositor: depositor.to_string(),
            // TODO
            amount: 0.0,
        };

        Ok(internal_deposit)
    }

    async fn proposal_deposit_v1beta1(&self, proposal_id: u64, depositor: &str) -> Result<InternalProposalDeposit, String> {
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryDepositRequest, QueryDepositResponse};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let deposit_request = QueryDepositRequest {
            proposal_id,
            depositor: depositor.to_string(),
        };
        let client = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .deposit(deposit_request)
            .await
            .map_err(|e| format!("{}", e))?;
        let deposit = client.into_inner();

        let internal_deposit = InternalProposalDeposit {
            depositor: depositor.to_string(),
            amount: 0.0,
        };

        Ok(internal_deposit)
    }
    /// Returns the deposit of given proposal by given depositor.
    pub async fn get_proposal_deposit_by_depositor(&self, proposal_id: u64, depositor: &str) -> Result<InternalProposalDeposit, String> {
        let items = if dbg!(self.config.sdk_version.minor) >= 46 {
            self.proposal_deposit_v1(proposal_id, depositor).await.ok()
        } else {
            None
        };

        let items = if let Some(items) = items {
            items
        } else {
            self.proposal_deposit_v1beta1(proposal_id, depositor)
                .await
                .map_err(|e| format!("Upstream error: {}", e))?
        };

        Ok(items)
    }

    /// Returns the tally of given proposal.
    pub async fn get_proposal_tally(&self, proposal_id: u64) -> Result<OutRestResponse<InternalProposalFinalTallyResult>, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/tally");

        let resp = self.rest_api_request::<ProposalTallyResp>(&path, &[]).await?;

        Ok(OutRestResponse::new(
            InternalProposalFinalTallyResult::try_from_with(resp.tally, |a| self.calc_amount_u128_to_f64(a))?,
            0,
        ))
    }

    /// Returns the votes of given proposal.
    pub async fn get_proposal_votes(&self, proposal_id: u64, config: PaginationConfig) -> Result<OutRestResponse<Vec<InternalProposalVote>>, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/votes");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<ProposalVotesResp>(&path, &query).await?;

        let mut proposal_votes = vec![];

        for proposal_vote in resp.votes {
            match proposal_vote.try_into() {
                Ok(proposal_vote) => proposal_votes.push(proposal_vote),
                Err(error) => tracing::error!("{error}"),
            }
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(proposal_votes, pages))
    }

    /// Returns the vote of given proposal by given voter.
    pub async fn get_proposal_vote_by_voter(&self, proposal_id: u64, voter: &str) -> Result<OutRestResponse<InternalProposalVote>, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/votes/{voter}");

        let resp = self.rest_api_request::<ProposalVoteByVoterResp>(&path, &[]).await?;

        let proposal_vote = resp.vote.try_into()?;

        Ok(OutRestResponse::new(proposal_vote, 0))
    }

    fn from(content: prost_types::Any) -> Self {
        todo!()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalVoteByVoterResp {
    /// Proposal vote.
    pub vote: ProposalVote,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalVotesResp {
    /// Array of proposal votes.
    pub votes: Vec<ProposalVote>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalProposalVote {
    /// Proposal ID. Eg: `34`
    pub proposal_id: u32,
    /// Proposal voter. Eg: `""`
    pub voter: String,
    /// Proposal vote option. Eg: `"VOTE_OPTION_UNSPECIFIED"`
    pub option: String,
    /// Array of proposal options.
    pub options: Vec<ProposalOption>,
}

impl TryFrom<ProposalVote> for InternalProposalVote {
    type Error = String;
    fn try_from(value: ProposalVote) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id: value
                .proposal_id
                .parse()
                .map_err(|_| format!("Cannot parse proposal id, '{}'.", value.proposal_id))?,
            voter: value.voter,
            option: value.option,
            options: value.options,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalOption {
    /// Proposal vote option. Eg: `"VOTE_OPTION_UNSPECIFIED"`
    pub option: String,
    /// Proposal vote option weight. Eg: `""`
    pub weight: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalTallyResp {
    /// Proposal tally.
    pub tally: ProposalFinalTallyResult,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalDepositByDepositorResp {
    /// Proposal deposit.
    pub deposit: ProposalDeposit,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalDepositsResp {
    /// Proposal deposits.
    pub deposits: Vec<ProposalDeposit>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalDeposit {
    /// Proposal depositor. Eg: `""`
    pub depositor: String,
    /// Amounts and denom deposited.
    pub amount: Vec<DenomAmount>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalProposalDeposit {
    /// Proposal depositor. Eg: `""`
    pub depositor: String,
    /// Amount deposited.
    pub amount: f64,
}

impl InternalProposalDeposit {
    fn new(value: ProposalDeposit, chain: &Chain) -> Result<Self, String> {
        Ok(Self {
            depositor: value.depositor,
            amount: match value.amount.get(0) {
                Some(da) => chain.calc_amount_u128_to_f64(
                    da.amount
                        .parse::<u128>()
                        .map_err(|_| "Cannot parse proposal deposit amount.".to_string())?,
                ),
                None => return Err("There is no proposal deposit.".to_string()),
            },
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalProposal {
    /// Proposal ID. Eg: `79`
    pub id: u64,
    /// Proposal content.
    pub messages: Vec<ProposalInfo>,
    /// Proposal status. Eg: `"Passed"`
    pub status: ProposalStatus,
    /// Proposal final tally result.
    pub final_tally_result: Option<InternalProposalFinalTallyResult>,
    /// Proposal submit timestamp in milliseconds.
    pub submit_time: Option<Timestamp>,
    /// Proposal deposit deadline timestamp in milliseconds.
    pub deposit_end_time: Option<Timestamp>,
    /// Proposal total deposit in the native coin of the chain..
    pub total_deposit: ChainAmountItem,
    /// Proposal voting start timestamp in milliseconds.
    pub voting_start_time: Option<Timestamp>,
    /// Proposal voting start timestamp in milliseconds.
    pub voting_end_time: Option<Timestamp>,

    pub metadata: Option<String>,
    // Since: cosmos-sdk 0.47
    pub title: String,
    pub summary: String,
    // Since: cosmos-sdk 0.47
    pub proposer: Option<String>,
    // Since: cosmos-sdk 0.48
    pub expedited: Option<bool>,
}

fn default_proposal_title() -> String {
    String::from("Unknown proposal.")
}

fn default_proposal_description() -> String {
    String::from("This proposal has no description.")
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalProposalFinalTallyResult {
    /// Number of `yes` votes. Eg: `"50"`
    pub yes_count: String,
    /// Number of `abstain` votes. Eg: `"35"`
    pub abstain_count: String,
    /// Number of `no` votes. Eg: `"12"`
    pub no_count: String,
    /// Number of `no with veto` votes.  Eg: `"7"`
    pub no_with_veto_count: String,
}

#[derive(Serialize, Debug)]
pub struct ProposalItem {
    /// Proposal ID.
    pub proposal_id: u64,
    /// Proposal Title.
    pub title: String,
    /// Proposal ID. Eg: `79`
    pub description: String,
    /// Voting start timestamp in milliseconds.
    pub time: Option<i64>,
    /// Proposal status.
    pub status: i32,
    // Content.
    pub content: serde_json::Value,
}
