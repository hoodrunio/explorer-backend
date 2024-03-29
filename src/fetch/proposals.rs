use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::str;
use tonic::transport::Endpoint;

use crate::{
    chain::Chain,
    database::ListDbResult,
    routes::{
        PaginationData,
        ProposalStatus,
        ChainAmountItem,
        PaginationDirection
    },
    fetch::{
        amount_util::TnrDecimal,
        cosmos::{
            base::query::v1beta1::{PageRequest, PageResponse},
            distribution::v1beta1::CommunityPoolSpendProposal,
            gov::{
                v1::{MsgExecLegacyContent, TallyResult as TallyResultV1},
                v1beta1::{TallyResult as TallyResultV1Beta1, TextProposal}
            },
            params::v1beta1::ParameterChangeProposal,
            upgrade::{
                v1beta1::SoftwareUpgradeProposal,
                v1beta1::MsgSoftwareUpgrade
            }
        },
        evmos::{
            erc20::v1::{RegisterCoinProposal, RegisterErc20Proposal, ToggleTokenConversionProposal},
            incentives::v1::RegisterIncentiveProposal,
        },
        gravity::v1::IbcMetadataProposal,
        ibc::core::client::v1::ClientUpdateProposal,
        kyve::global::v1beta1::MsgUpdateParams as KyveMsgUpdateParams,
        osmosis::{
            poolincentives::v1beta1::UpdatePoolIncentivesProposal,
            superfluid::v1beta1::{RemoveSuperfluidAssetsProposal, SetSuperfluidAssetsProposal},
            txfees::v1beta1::UpdateFeeTokenProposal
        },
        quicksilver::interchainstaking::v1::RegisterZoneProposal,
        umee::leverage::v1::MsgGovUpdateRegistry,
        cosmwasm::wasm::v1::StoreCodeProposal,
    },
    utils::ts_to_ms,
};

use prost::Message;
use crate::fetch::lavanet::lava::plans::PlansAddProposal;
use crate::fetch::lavanet::lava::spec::SpecAddProposal;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProposalInfo {
    pub title: String,
    pub description: String,
    pub type_url: String,
    pub content: serde_json::Value,
}

impl From<prost_wkt_types::Any> for ProposalInfo {
    fn from(content: prost_wkt_types::Any) -> ProposalInfo {
        let mut content = content;

        if content.type_url.ends_with("cosmos.gov.v1.MsgExecLegacyContent") {
            let decoded = MsgExecLegacyContent::decode(content.value.as_ref()).unwrap();
            content = decoded.content.unwrap();
        }

        let (title, description, content_value) = match content.type_url.as_str() {
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
            "/kyve.global.v1beta1.MsgUpdateParams" => {
                let value = KyveMsgUpdateParams::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(value).unwrap();
                ("".to_string(), "".to_string(), content)
            }
            "/cosmwasm.wasm.v1.StoreCodeProposal" => {
                let value = StoreCodeProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/osmosis.txfees.v1beta1.UpdateFeeTokenProposal" => {
                let value = UpdateFeeTokenProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/osmosis.superfluid.v1beta1.SetSuperfluidAssetsProposal" => {
                let value = SetSuperfluidAssetsProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/osmosis.superfluid.v1beta1.RemoveSuperfluidAssetsProposal" => {
                let value = RemoveSuperfluidAssetsProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/cosmos.upgrade.v1beta1.MsgSoftwareUpgrade" => {
                let value = MsgSoftwareUpgrade::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                ("".to_string(), "".to_string(), content)

            }
            "/lavanet.lava.plans.PlansAddProposal" => {
                let value = PlansAddProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }
            "/lavanet.lava.spec.SpecAddProposal" => {
                let value = SpecAddProposal::decode(content.value.as_ref()).unwrap();
                let content = serde_json::to_value(&value).unwrap();
                (value.title, value.description, content)
            }

            _other => (String::from(""), String::from(""), serde_json::Value::Null),
        };
        ProposalInfo {
            title,
            description,
            type_url: content.type_url,
            content: content_value,
        }
    }
}

impl Into<PageRequest> for PaginationData {
    fn into(self) -> PageRequest {
        PageRequest {
            key: self.cursor.map(|b| base64::decode(b).unwrap_or_default().to_vec()).unwrap_or_default(),
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
            Some(base64::encode(value.next_key))
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

impl PaginationData {
    fn from_grpc_pagin_resp(value: PageResponse, limit: Option<u64>) -> Self {
        let cursor = if !value.next_key.is_empty() {
            Some(base64::encode(value.next_key))
        } else {
            None
        };
        Self {
            cursor,
            offset: None,
            limit,
            direction: Some(PaginationDirection::Next),
        }
    }
}

impl Chain {
    async fn get_proposals_v1(&self, status: &str, config: PaginationData) -> Result<ListDbResult<ProposalItem>, String> {
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, Proposal, QueryProposalsRequest};
        let limit = config.limit;
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let pagination = PageRequest {
            reverse: true,
            ..config.into()
        };
        let proposal_request = QueryProposalsRequest {
            proposal_status: status.parse().unwrap(),
            voter: "".to_string(),
            depositor: "".to_string(),
            pagination: Some(pagination),
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
                let ProposalInfo {
                    title,
                    description,
                    type_url,
                    content,
                } = content.into();
                let Proposal { id, submit_time, status, .. } = proposal;
                let proposal_item = ProposalItem {
                    proposal_id: id,
                    title,
                    description,
                    time: submit_time.map(|t| t.seconds),
                    status,
                    type_url,
                    content,
                };

                items.push(proposal_item);
            };
        }

        Ok(ListDbResult {
            data: items,
            pagination: proposals
                .pagination
                .map(|p| PaginationData::from_grpc_pagin_resp(p, limit))
                .unwrap_or_default(),
        })
    }
    async fn get_proposals_v1beta1(&self, status: &str, config: PaginationData) -> Result<ListDbResult<ProposalItem>, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, Proposal, QueryProposalsRequest};
        let limit = config.limit;
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let pagination = PageRequest {
            reverse: true,
            ..config.into()
        };
        let proposal_request = QueryProposalsRequest {
            proposal_status: status.parse().unwrap(),
            voter: "".to_string(),
            depositor: "".to_string(),
            pagination: Some(pagination),
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
                let ProposalInfo {
                    title,
                    description,
                    type_url,
                    content,
                } = content.into();
                let proposal_item = ProposalItem {
                    proposal_id,
                    title,
                    description,
                    time: submit_time.map(|t| t.seconds),
                    status,
                    type_url,
                    content,
                };

                items.push(proposal_item);
            };
        }

        Ok(ListDbResult {
            data: items,
            pagination: proposals
                .pagination
                .map(|p| PaginationData::from_grpc_pagin_resp(p, limit))
                .unwrap_or_default(),
        })
    }
    /// Returns all the proposals in voting period.
    pub async fn get_proposals_by_status(&self, status: ProposalStatus, config: PaginationData) -> Result<ListDbResult<ProposalItem>, String> {
        let status_id = status.get_id().to_string();
        let items = if self.config.sdk_version.minor >= 46 {
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
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryProposalRequest};
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
        let final_tally_result = self.get_proposal_tally(proposal_id).await.ok();

        let messages = proposal.messages.into_iter().map(|m| m.into()).collect();
        let total_deposit_string_amount = proposal.total_deposit.iter().map(|d| d.amount.clone()).collect();
        let total_deposit = self.string_amount_parser(total_deposit_string_amount, None).await.unwrap_or_default();

        let internal_proposal = InternalProposal {
            id: proposal.id,
            messages,
            status: ProposalStatus::from_id(proposal.status),
            final_tally_result,
            total_deposit,
            submit_time: proposal.submit_time.map(|ts| ts_to_ms(&ts.to_string()).unwrap_or_default()),
            deposit_end_time: proposal.deposit_end_time.map(|ts| ts_to_ms(&ts.to_string()).unwrap_or_default()),
            voting_start_time: proposal.voting_start_time.map(|ts| ts_to_ms(&ts.to_string()).unwrap_or_default()),
            voting_end_time: proposal.voting_end_time.map(|ts| ts_to_ms(&ts.to_string()).unwrap_or_default()),
            metadata: Some(proposal.metadata),
            title: proposal.title,
            summary: proposal.summary,
            proposer: Some(proposal.proposer),
            expedited: Some(proposal.expedited),
        };

        Ok(internal_proposal)
    }
    async fn get_proposal_details_v1beta1(&self, proposal_id: u64) -> Result<InternalProposal, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, QueryProposalRequest};
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

        let prop_info: Option<ProposalInfo> = proposal.content.map(|c| c.into());

        let (title, summary) = prop_info
            .clone()
            .map_or_else(|| (String::from(""), String::from("")), |p| (p.title.clone(), p.description));
        let mut messages = vec![];
        if let Some(p) = prop_info {
            messages.push(p);
        }

        let final_tally_result = self.get_proposal_tally(proposal_id).await.ok();

        let total_deposit_string_amount = proposal.total_deposit.iter().map(|d| d.amount.clone()).collect();
        let total_deposit = self.string_amount_parser(total_deposit_string_amount, None).await.unwrap_or_default();

        let internal_proposal = InternalProposal {
            id: proposal_id,
            messages,
            status: ProposalStatus::from_id(proposal.status),
            final_tally_result,
            submit_time: proposal.submit_time.map(|ts| ts_to_ms(&ts.to_string()).unwrap_or_default()),
            deposit_end_time: proposal.deposit_end_time.map(|ts| ts_to_ms(&ts.to_string()).unwrap_or_default()),
            total_deposit,
            voting_start_time: proposal.voting_start_time.map(|ts| ts_to_ms(&ts.to_string()).unwrap_or_default()),
            voting_end_time: proposal.voting_end_time.map(|ts| ts_to_ms(&ts.to_string()).unwrap_or_default()),
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
        let items = if self.config.sdk_version.minor >= 46 {
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
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryDepositsRequest};
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

        let internal_deposits = join_all(deposits.deposits.iter().map(|d| async move {
            let string_amount = d.amount.iter().map(|d| d.amount.clone()).collect();
            let amount = self.string_amount_parser(string_amount, None).await.unwrap_or_default();

            InternalProposalDeposit {
                depositor: d.depositor.clone(),
                amount,
            }
        }))
        .await;

        Ok(ListDbResult {
            data: internal_deposits,
            pagination: deposits.pagination.map(|p| p.into()).unwrap_or_default(),
        })
    }

    async fn proposal_deposits_v1beta1(&self, proposal_id: u64, config: PaginationData) -> Result<ListDbResult<InternalProposalDeposit>, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, QueryDepositsRequest};
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
        let internal_deposits = join_all(deposits.deposits.iter().map(|d| async move {
            let string_amount = d.amount.iter().map(|d| d.amount.clone()).collect();
            let amount = self.string_amount_parser(string_amount, None).await.unwrap_or_default();

            InternalProposalDeposit {
                depositor: d.depositor.clone(),
                amount,
            }
        }))
        .await;

        Ok(ListDbResult {
            data: internal_deposits,
            pagination: deposits.pagination.map(|p| p.into()).unwrap_or_default(),
        })
    }
    /// Returns the deposits of given proposal.
    pub async fn get_proposal_deposits(&self, proposal_id: u64, config: PaginationData) -> Result<ListDbResult<InternalProposalDeposit>, String> {
        let items = if self.config.sdk_version.minor >= 46 {
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
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryDepositRequest};
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
        let deposit = deposit.deposit.ok_or_else(|| String::from("Deposit not found"))?;

        let string_amount = deposit.amount.iter().map(|d| d.amount.clone()).collect();
        let amount = self.string_amount_parser(string_amount, None).await.unwrap_or_default();

        let internal_deposit = InternalProposalDeposit {
            depositor: depositor.to_string(),
            // TODO
            amount,
        };

        Ok(internal_deposit)
    }

    async fn proposal_deposit_v1beta1(&self, proposal_id: u64, depositor: &str) -> Result<InternalProposalDeposit, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, QueryDepositRequest};
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
        let deposit = deposit.deposit.ok_or_else(|| String::from("Deposit not found"))?;

        let string_amount = deposit.amount.iter().map(|d| d.amount.clone()).collect();
        let amount = self.string_amount_parser(string_amount, None).await.unwrap_or_default();

        let internal_deposit = InternalProposalDeposit {
            depositor: depositor.to_string(),
            amount,
        };

        Ok(internal_deposit)
    }
    /// Returns the deposit of given proposal by given depositor.
    pub async fn get_proposal_deposit_by_depositor(&self, proposal_id: u64, depositor: &str) -> Result<InternalProposalDeposit, String> {
        let items = if self.config.sdk_version.minor >= 46 {
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

    async fn proposal_tally_v1(&self, proposal_id: u64) -> Result<InternalProposalFinalTallyResult, String> {
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryTallyResultRequest};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let tally_request = QueryTallyResultRequest { proposal_id };
        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .tally_result(tally_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let tally_resp = resp.into_inner();
        let tally = tally_resp.tally.ok_or_else(|| String::from("Tally not found"))?;

        let internal_proposal_final_tally_result = InternalProposalFinalTallyResult::from_raw_tally_result(self, tally.into()).await;

        Ok(internal_proposal_final_tally_result)
    }

    async fn proposal_tally_v1beta1(&self, proposal_id: u64) -> Result<InternalProposalFinalTallyResult, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, QueryTallyResultRequest};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let tally_request = QueryTallyResultRequest { proposal_id };
        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .tally_result(tally_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let tally_resp = resp.into_inner();
        let tally = tally_resp.tally.ok_or_else(|| String::from("Tally not found"))?;

        let internal_proposal_final_tally_result = InternalProposalFinalTallyResult::from_raw_tally_result(self, tally.into()).await;

        Ok(internal_proposal_final_tally_result)
    }

    /// Returns the tally of given proposal.
    pub async fn get_proposal_tally(&self, proposal_id: u64) -> Result<InternalProposalFinalTallyResult, String> {
        let items = if self.config.sdk_version.minor >= 46 {
            self.proposal_tally_v1(proposal_id).await.ok()
        } else {
            None
        };

        let items = if let Some(items) = items {
            items
        } else {
            self.proposal_tally_v1beta1(proposal_id)
                .await
                .map_err(|e| format!("Upstream error: {}", e))?
        };

        Ok(items)
    }

    async fn proposal_votes_v1(&self, proposal_id: u64, config: PaginationData) -> Result<ListDbResult<InternalProposalVote>, String> {
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryVotesRequest};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let votes_request = QueryVotesRequest {
            proposal_id,
            pagination: Some(config.into()),
        };
        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .votes(votes_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let votes_resp = resp.into_inner();
        let votes = votes_resp.votes;

        let internal_proposal_votes = votes
            .iter()
            .map(|v| {
                let options = v
                    .options
                    .iter()
                    .map(|o| ProposalOption {
                        option: o.option,
                        weight: o.weight.clone(),
                    })
                    .collect();

                InternalProposalVote {
                    proposal_id,
                    voter: v.voter.clone(),
                    option: String::default(),
                    options,
                    metadata: Some(v.metadata.clone()),
                }
            })
            .collect();

        Ok(ListDbResult {
            data: internal_proposal_votes,
            pagination: votes_resp.pagination.map(|p| p.into()).unwrap_or_default(),
        })
    }

    async fn proposal_votes_v1beta1(&self, proposal_id: u64, config: PaginationData) -> Result<ListDbResult<InternalProposalVote>, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, QueryVotesRequest};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let votes_request = QueryVotesRequest {
            proposal_id,
            pagination: Some(config.into()),
        };
        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .votes(votes_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let votes_resp = resp.into_inner();
        let votes = votes_resp.votes;

        let internal_proposal_votes = votes
            .iter()
            .map(|v| {
                let options = v
                    .options
                    .iter()
                    .map(|o| ProposalOption {
                        option: o.option,
                        weight: o.weight.clone(),
                    })
                    .collect();

                InternalProposalVote {
                    proposal_id,
                    voter: v.voter.clone(),
                    option: String::default(),
                    options,
                    metadata: None,
                }
            })
            .collect();

        Ok(ListDbResult {
            data: internal_proposal_votes,
            pagination: votes_resp.pagination.map(|p| p.into()).unwrap_or_default(),
        })
    }
    /// Returns the votes of given proposal.
    pub async fn get_proposal_votes(&self, proposal_id: u64, config: PaginationData) -> Result<ListDbResult<InternalProposalVote>, String> {
        let items = if self.config.sdk_version.minor >= 46 {
            self.proposal_votes_v1(proposal_id, config.clone()).await.ok()
        } else {
            None
        };

        let items = if let Some(items) = items {
            items
        } else {
            self.proposal_votes_v1beta1(proposal_id, config)
                .await
                .map_err(|e| format!("Upstream error: {}", e))?
        };

        Ok(items)
    }

    async fn proposal_vote_by_voter_v1(&self, proposal_id: u64, voter: &str) -> Result<InternalProposalVote, String> {
        use crate::fetch::cosmos::gov::v1::{query_client::QueryClient, QueryVoteRequest};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let vote_request = QueryVoteRequest {
            proposal_id,
            voter: voter.to_string(),
        };
        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .vote(vote_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let vote_resp = resp.into_inner();
        let vote = vote_resp.vote.ok_or_else(|| String::from("Vote not found"))?;

        let internal_proposal_vote = InternalProposalVote {
            proposal_id,
            voter: vote.voter.clone(),
            option: String::default(),
            options: vote
                .options
                .iter()
                .map(|o| ProposalOption {
                    option: o.option,
                    weight: o.weight.clone(),
                })
                .collect(),
            metadata: Some(vote.metadata.clone()),
        };

        Ok(internal_proposal_vote)
    }
    async fn proposal_vote_by_voter_v1beta1(&self, proposal_id: u64, voter: &str) -> Result<InternalProposalVote, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, QueryVoteRequest};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let vote_request = QueryVoteRequest {
            proposal_id,
            voter: voter.to_string(),
        };
        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .vote(vote_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let vote_resp = resp.into_inner();
        let vote = vote_resp.vote.ok_or_else(|| String::from("Vote not found"))?;

        let internal_proposal_vote = InternalProposalVote {
            proposal_id,
            voter: vote.voter.clone(),
            option: String::default(),
            options: vote
                .options
                .iter()
                .map(|o| ProposalOption {
                    option: o.option,
                    weight: o.weight.clone(),
                })
                .collect(),
            metadata: None,
        };

        Ok(internal_proposal_vote)
    }
    /// Returns the vote of given proposal by given voter.
    pub async fn get_proposal_vote_by_voter(&self, proposal_id: u64, voter: &str) -> Result<InternalProposalVote, String> {
        let items = if self.config.sdk_version.minor >= 46 {
            self.proposal_vote_by_voter_v1(proposal_id, voter).await.ok()
        } else {
            None
        };

        let items = if let Some(items) = items {
            items
        } else {
            self.proposal_vote_by_voter_v1beta1(proposal_id, voter)
                .await
                .map_err(|e| format!("Upstream error: {}", e))?
        };

        Ok(items)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalProposalVote {
    /// Proposal ID. Eg: `34`
    pub proposal_id: u64,
    /// Proposal voter. Eg: `""`
    pub voter: String,
    /// Proposal vote option. Eg: `"VOTE_OPTION_UNSPECIFIED"`
    pub option: String,
    /// Array of proposal options.
    pub options: Vec<ProposalOption>,
    // metadata is any  arbitrary metadata to attached to the vote.
    pub metadata: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalOption {
    /// Proposal vote option. Eg: `"VOTE_OPTION_UNSPECIFIED"`
    pub option: i32,
    /// Proposal vote option weight. Eg: `""`
    pub weight: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalProposalDeposit {
    /// Proposal depositor. Eg: `""`
    pub depositor: String,
    /// Amount deposited.
    pub amount: ChainAmountItem,
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
    pub submit_time: Option<i64>,
    /// Proposal deposit deadline timestamp in milliseconds.
    pub deposit_end_time: Option<i64>,
    /// Proposal total deposit in the native coin of the chain..
    pub total_deposit: ChainAmountItem,
    /// Proposal voting start timestamp in milliseconds.
    pub voting_start_time: Option<i64>,
    /// Proposal voting start timestamp in milliseconds.
    pub voting_end_time: Option<i64>,

    pub metadata: Option<String>,
    // Since: cosmos-sdk 0.47
    pub title: String,
    pub summary: String,
    // Since: cosmos-sdk 0.47
    pub proposer: Option<String>,
    // Since: cosmos-sdk 0.48
    pub expedited: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct InternalProposalFinalTallyResult {
    /// Number of `yes` votes. Eg: `"50"`
    pub raw_yes_count: String,
    pub yes_ratio: TnrDecimal,
    pub yes_amount: ChainAmountItem,
    /// Number of `abstain` votes. Eg: `"35"`
    pub raw_abstain_count: String,
    pub abstain_ratio: TnrDecimal,
    pub abstain_amount: ChainAmountItem,
    /// Number of `no` votes. Eg: `"12"`
    pub raw_no_count: String,
    pub no_ratio: TnrDecimal,
    pub no_amount: ChainAmountItem,
    /// Number of `no with veto` votes.  Eg: `"7"`
    pub raw_no_with_veto_count: String,
    pub no_with_veto_ratio: TnrDecimal,
    pub no_with_veto_amount: ChainAmountItem,
}

impl InternalProposalFinalTallyResult {
    pub async fn from_raw_tally_result(chain: &Chain, raw_tally_result: RawProposalFinalTallyResult) -> Self {
        let yes_amount = chain
            .string_amount_parser(raw_tally_result.raw_yes.clone(), None)
            .await
            .unwrap_or_default();
        let abstain_amount = chain
            .string_amount_parser(raw_tally_result.raw_abstain.clone(), None)
            .await
            .unwrap_or_default();
        let no_amount = chain
            .string_amount_parser(raw_tally_result.raw_no.clone(), None)
            .await
            .unwrap_or_default();
        let no_with_veto_amount = chain
            .string_amount_parser(raw_tally_result.raw_no_with_veto.clone(), None)
            .await
            .unwrap_or_default();

        let total_voting_power = yes_amount.amount + abstain_amount.amount + no_amount.amount + no_with_veto_amount.amount;

        let yes_ratio = yes_amount.amount.checked_div(total_voting_power).unwrap_or_default();
        let abstain_ratio = abstain_amount.amount.checked_div(total_voting_power).unwrap_or_default();
        let no_ratio = no_amount.amount.checked_div(total_voting_power).unwrap_or_default();
        let no_with_veto_ratio = no_with_veto_amount.amount.checked_div(total_voting_power).unwrap_or_default();

        InternalProposalFinalTallyResult {
            raw_yes_count: raw_tally_result.raw_yes,
            yes_amount,
            yes_ratio,
            raw_abstain_count: raw_tally_result.raw_abstain,
            abstain_amount,
            abstain_ratio,
            raw_no_count: raw_tally_result.raw_no,
            no_amount,
            no_ratio,
            raw_no_with_veto_count: raw_tally_result.raw_no_with_veto,
            no_with_veto_amount,
            no_with_veto_ratio,
        }
    }
}

pub struct RawProposalFinalTallyResult {
    pub raw_yes: String,
    pub raw_abstain: String,
    pub raw_no: String,
    pub raw_no_with_veto: String,
}

impl From<TallyResultV1Beta1> for RawProposalFinalTallyResult {
    fn from(tally_result: TallyResultV1Beta1) -> Self {
        Self {
            raw_yes: tally_result.yes,
            raw_abstain: tally_result.abstain,
            raw_no: tally_result.no,
            raw_no_with_veto: tally_result.no_with_veto,
        }
    }
}

impl From<TallyResultV1> for RawProposalFinalTallyResult {
    fn from(tally_result: TallyResultV1) -> Self {
        Self {
            raw_yes: tally_result.yes_count,
            raw_abstain: tally_result.abstain_count,
            raw_no: tally_result.no_count,
            raw_no_with_veto: tally_result.no_with_veto_count,
        }
    }
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
    pub type_url: String,
    // Content.
    pub content: serde_json::Value,
}
