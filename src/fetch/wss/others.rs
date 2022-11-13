use serde::{Deserialize, Serialize};

use crate::fetch::rest::requests::RPCSuccessResponse;

use super::{
    new_blocks::{NewBlock, UnparsedEventAttribute},
    tx::Tx,
};

pub type SocketResponse<T> = RPCSuccessResponse<SubscribeResult<T>>;

#[derive(Deserialize, Serialize, Debug)]
pub struct SubscribeResult<T> {
    pub data: Option<SubscribeData<T>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SubscribeData<T> {
    pub value: T,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    #[serde(rename = "acknowledge_packet")]
    AcknowledgePacket { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "axelar.axelarnet.v1beta1.AxelarTransferCompleted")]
    AxelarAxelarnetAxelarTransferCompleted { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "axelar.axelarnet.v1beta1.FeeCollected")]
    AxelarAxelarnetFeeCollected { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "axelar.axelarnet.v1beta1.IBCTransferCompleted")]
    AxelarAxelarnetIBCTransferCompleted { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "axelar.evm.v1beta1.ConfirmDepositStarted")]
    AxelarEvmConfirmDepositStarted { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "axelar.evm.v1beta1.EVMEventConfirmed")]
    AxelarEvmEVMEventConfirmed { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "axelar.evm.v1beta1.MintCommand")]
    AxelarEvmMintCommand { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "axelar.multisig.v1beta1.SignatureSubmitted")]
    AxelarMultisigSignatureSubmitted { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "axelar.multisig.v1beta1.SigningStarted")]
    AxelarMultisigSigningStarted { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "axelar.vote.v1beta1.Voted")]
    AxelarVoteVoted { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "burn")]
    Burn { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "coin_received")]
    CoinReceived { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "coin_spent")]
    CoinSpent { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "coinbase")]
    Coinbase { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "commission")]
    Commission { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "convert_erc20")]
    ConvertErc20 { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "cosmos.authz.v1beta1.EventGrant")]
    CosmosAuthzEventGrant { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "create_validator")]
    CreateValidator { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "delegate")]
    Delegate { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "depositConfirmation")]
    DepositConfirmation { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "distribute_dev_revenue")]
    DistributeDevRevenue { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "ethereum_tx")]
    EthereumTx { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "eventConfirmation")]
    EventConfirmation { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "fungible_token_packet")]
    FungibleTokenPacket { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "ibc_transfer")]
    IbcTransfer { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "link")]
    Link { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "message")]
    Message { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "mint")]
    Mit { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "payfordata")]
    Payfordata { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "proposal_vote")]
    ProposalVote { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "proposer_reward")]
    ProposerReward { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "recv_packet")]
    RecvPacket { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "redelegate")]
    Redelegate { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "rewards")]
    Rewards { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "send_packet")]
    SendPacket { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "set_feegrant")]
    SetFeegrant { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "sign")]
    Sign { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "timeout")]
    Timeout { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "timeout_packet")]
    TimeoutPacket { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "transfer")]
    Transfer { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "tx")]
    Tx { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "tx_log")]
    TxLog { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "unbond")]
    Unbond { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "update_client")]
    UpdateClient { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "use_feegrant")]
    UseFeegrant { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "wasm")]
    Wasm { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "withdraw_commission")]
    WithdrawCommission { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "withdraw_rewards")]
    WithdrawRewards { attributes: Vec<UnparsedEventAttribute> },
    #[serde(rename = "write_acknowledgement")]
    WriteAcknowledgement { attributes: Vec<UnparsedEventAttribute> },
}
