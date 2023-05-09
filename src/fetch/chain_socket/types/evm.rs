use futures::future::join_all;
use serde::{Deserialize, Serialize};

use crate::{
    chain::Chain,
    database::{DatabaseTR, EvmPollForDb, EvmPollParticipantForDb},
    fetch::{blocks::CosmosEvent, evm::PollStatus, transactions::InternalTransaction},
    routes::TNRAppError,
};

use super::tx::TXMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfirmDepositStarted {
    chain: String,
    participants: PollParticipants,
    tx_id: String,
    evm_deposit_address: String,
    action: String,
    height: String,
}

impl ConfirmDepositStarted {
    pub fn from_tx_events(ev: TXMap) -> Self {
        let participants = serde_json::from_str(ev["axelar.evm.v1beta1.ConfirmDepositStarted.participants"].get(0).unwrap()).unwrap();

        Self {
            chain: ev["axelar.evm.v1beta1.ConfirmDepositStarted.chain"].get(0).unwrap().to_string(),
            participants,
            tx_id: ev["axelar.evm.v1beta1.ConfirmDepositStarted.tx_id"].get(0).unwrap().to_string(),
            evm_deposit_address: ev["axelar.evm.v1beta1.ConfirmDepositStarted.deposit_address"].get(0).unwrap().to_string(),
            action: ev["message.action"].get(0).unwrap().to_string(),
            height: ev["tx.height"].get(0).unwrap().to_string(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfirmGatewayTxStartedEvents {
    chain: String,
    participants: PollParticipants,
    tx_id: String,
    height: String,
    message_action: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollParticipants {
    poll_id: String,
    participants: Vec<String>,
}

impl ConfirmGatewayTxStartedEvents {
    pub fn from_tx_events(ev: TXMap) -> Self {
        let participants = serde_json::from_str(ev["axelar.evm.v1beta1.ConfirmGatewayTxStarted.participants"].get(0).unwrap()).unwrap();

        Self {
            chain: ev["axelar.evm.v1beta1.ConfirmGatewayTxStarted.chain"].get(0).unwrap().to_string(),
            participants,
            tx_id: ev["axelar.evm.v1beta1.ConfirmGatewayTxStarted.tx_id"].get(0).unwrap().to_string(),
            message_action: ev["message.action"].get(0).unwrap().to_string(),
            height: ev["tx.height"].get(0).unwrap().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfirmKeyTransferStartedEvents {
    chain: String,
    participants: PollParticipants,
    tx_id: String,
    message_action: String,
    height: String,
}

impl ConfirmKeyTransferStartedEvents {
    pub fn from_tx_events(ev: TXMap) -> Self {
        let participants = serde_json::from_str(ev["axelar.evm.v1beta1.ConfirmKeyTransferStarted.participants"].get(0).unwrap()).unwrap();

        Self {
            chain: ev["axelar.evm.v1beta1.ConfirmKeyTransferStarted.chain"].get(0).unwrap().to_string(),
            participants,
            tx_id: ev["axelar.evm.v1beta1.ConfirmKeyTransferStarted.tx_id"].get(0).unwrap().to_string(),
            message_action: ev["message.action"].get(0).unwrap().to_string(),
            height: ev["tx.height"].get(0).unwrap().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewPollEvent {
    chain: String,
    poll_participants: PollParticipants,
    tx_id: String,
    height: u64,
    evm_deposit_address: String,
    message_action: String,
}

impl NewPollEvent {
    pub async fn get_evm_poll_item(&self, chain: &Chain) -> Result<EvmPollItem, TNRAppError> {
        let tx_height = self.height;
        let chain_name = self.chain.clone();
        let action_name = self.message_action.clone();
        let poll_participants = self.poll_participants.clone();
        let tx_id = self.tx_id.clone();
        let deposit_address = self.evm_deposit_address.clone();

        EvmPollItem::new(
            &EvmPollItemEventParams {
                chain: chain_name,
                deposit_address,
                tx_height,
                action_name,
                poll_participants,
                tx_id,
            },
            chain,
        )
        .await
    }
}

impl Default for NewPollEvent {
    fn default() -> Self {
        Self {
            chain: "".to_string(),
            poll_participants: PollParticipants {
                poll_id: String::default(),
                participants: vec![],
            },
            tx_id: "".to_string(),
            evm_deposit_address: "".to_string(),
            message_action: "".to_string(),
            height: 0,
        }
    }
}

impl From<ConfirmDepositStarted> for NewPollEvent {
    fn from(e: ConfirmDepositStarted) -> Self {
        Self {
            chain: e.chain,
            poll_participants: e.participants,
            tx_id: e.tx_id,
            evm_deposit_address: e.evm_deposit_address,
            message_action: e.action,
            height: e.height.parse::<u64>().unwrap_or(0),
        }
    }
}
impl From<ConfirmGatewayTxStartedEvents> for NewPollEvent {
    fn from(e: ConfirmGatewayTxStartedEvents) -> Self {
        Self {
            chain: e.chain,
            poll_participants: e.participants,
            tx_id: e.tx_id,
            message_action: e.message_action,
            height: e.height.parse::<u64>().unwrap_or(0),
            ..Default::default()
        }
    }
}
impl From<ConfirmKeyTransferStartedEvents> for NewPollEvent {
    fn from(e: ConfirmKeyTransferStartedEvents) -> Self {
        Self {
            chain: e.chain,
            poll_participants: e.participants,
            tx_id: e.tx_id,
            message_action: e.message_action,
            height: e.height.parse::<u64>().unwrap_or(0),
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollVoteEvent {
    pub poll_state: String,
    pub hash: String,
}

impl PollVoteEvent {
    pub fn from_tx_events(ev: TXMap) -> Self {
        Self {
            poll_state: ev["axelar.vote.v1beta1.Voted.state"].get(0).unwrap().to_string(),
            hash: ev.get("tx.hash").unwrap().get(0).unwrap().to_string(),
        }
    }

    pub async fn fetch_tx(&self, chain: &Chain, tx_hash: String) -> Result<InternalTransaction, TNRAppError> {
        let internal_tx = match chain.get_tx_by_hash(&tx_hash).await {
            Ok(res) => res.value,
            Err(e) => {
                tracing::error!("tx could not fetched retrying 1 {}", e);
                match chain.get_tx_by_hash(&tx_hash).await {
                    Ok(res) => res.value,
                    Err(e) => {
                        tracing::error!("tx could not fetched retrying 2  {}", e);
                        match chain.get_tx_by_hash(&tx_hash).await {
                            Ok(res) => res.value,
                            Err(e) => {
                                tracing::error!("tx could not fetched  {}", e);
                                return Err(TNRAppError::from(e));
                            }
                        }
                    }
                }
            }
        };

        Ok(internal_tx)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EvmPollItem {
    pub tx_height: u64,
    pub action: String,
    pub poll_id: String,
    pub chain_name: String,
    pub status: String,
    pub evm_tx_id: String,
    pub evm_deposit_address: String,
    pub participants_operator_address: Vec<String>,
    pub time: u64,
}

impl EvmPollItem {
    async fn new(params: &EvmPollItemEventParams, chain: &Chain) -> Result<Self, TNRAppError> {
        let poll_info = params.poll_participants.clone();

        let tx_height = params.tx_height;
        let time = match chain.get_block_by_height(Some(tx_height)).await {
            Ok(res) => res.value.time as u64,
            Err(_) => 0,
        };

        let chain_name = str::replace(&params.chain, "\"", "");
        let evm_tx_id = chain.convert_to_evm_hex(&params.tx_id).unwrap_or(String::from(""));
        let evm_deposit_address = chain.convert_to_evm_hex(&params.deposit_address).unwrap_or(String::from(""));
        let action = String::from(&params.action_name);

        Ok(Self {
            poll_id: poll_info.poll_id.clone(),
            status: String::from("Pending"),
            participants_operator_address: poll_info.participants.clone(),
            evm_deposit_address,
            action,
            evm_tx_id,
            chain_name,
            time,
            tx_height,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
struct EvmPollItemEventParams {
    pub tx_height: u64,
    pub chain: String,
    pub action_name: String,
    pub poll_participants: PollParticipants,
    pub tx_id: String,
    pub deposit_address: String,
}

impl From<EvmPollItem> for EvmPollForDb {
    fn from(value: EvmPollItem) -> Self {
        EvmPollForDb {
            timestamp: value.time,
            tx_height: value.tx_height,
            poll_id: value.poll_id.clone(),
            action: value.action.clone(),
            status: value.status.clone(),
            evm_tx_id: value.evm_tx_id.clone(),
            chain_name: value.chain_name.clone(),
            evm_deposit_address: value.evm_deposit_address,
        }
    }
}

impl EvmPollItem {
    pub async fn upsert_participants(&self, db: &DatabaseTR) -> Result<(), String> {
        let participants: Vec<EvmPollParticipantForDb> = self
            .participants_operator_address
            .iter()
            .map(|address| EvmPollParticipantForDb::from_info(address.clone(), self.poll_id.clone(), self.chain_name.clone()))
            .collect();

        let mut db_jobs = vec![];
        for participant in participants {
            db_jobs.push(async move { db.upsert_evm_poll_participant(participant).await });
        }
        join_all(db_jobs).await;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum EvmPollVote {
    UnSubmit,
    Yes,
    No,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EvmPollBlockInfo {
    pub events: Vec<CosmosEvent>,
}

impl EvmPollBlockInfo {
    fn extract_evm_poll_info(&self, event: &CosmosEvent, status: PollStatus) -> AxelarCompletedPoll {
        let mut poll_id: String = String::from("");
        let mut chain: String = String::from("");
        let mut tx_id: String = String::from("");

        for attribute in event.attributes.clone() {
            if attribute.key == "poll_id" {
                poll_id = attribute.value.clone().replace('"', "");
            };
            if attribute.key == "chain" {
                chain = attribute.value.clone().replace('"', "");
            };
            if attribute.key == "tx_id" {
                tx_id = attribute.value.clone();
            };
        }

        AxelarCompletedPoll {
            chain,
            poll_id,
            tx_id,
            poll_status: status,
        }
    }

    pub fn extract_evm_poll_completed_events(&self) -> Option<Vec<AxelarCompletedPoll>> {
        let end_block_events = &self.events;
        if end_block_events.is_empty() {
            return None;
        };
        let mut poll_completed_axelar_polls: Vec<AxelarCompletedPoll> = vec![];

        for event in end_block_events {
            if event.r#type == "axelar.evm.v1beta1.PollCompleted" {
                let completed_axelar_poll_info = self.extract_evm_poll_info(event, PollStatus::Completed);
                let ignore = poll_completed_axelar_polls
                    .clone()
                    .into_iter()
                    .any(|poll| poll.poll_id == completed_axelar_poll_info.poll_id);

                if !ignore {
                    poll_completed_axelar_polls.push(completed_axelar_poll_info);
                };
            };
            if event.r#type == "axelar.evm.v1beta1.NoEventsConfirmed" {
                let axelar_poll_info = self.extract_evm_poll_info(event, PollStatus::Failed);
                poll_completed_axelar_polls.push(axelar_poll_info);
            };
        }

        if poll_completed_axelar_polls.is_empty() {
            return None;
        }

        Some(poll_completed_axelar_polls)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct AxelarCompletedPoll {
    pub chain: String,
    pub poll_id: String,
    pub tx_id: String,
    pub poll_status: PollStatus,
}
