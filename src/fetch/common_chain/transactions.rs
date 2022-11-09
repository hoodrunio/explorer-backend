use serde::{Deserialize, Serialize};


use super::others::{DenomAmount, Pagination, PublicKey};

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsResp {
    pub txs: Vec<TxsTransaction>,
    pub tx_responses: Vec<TxResponse>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransaction {
    /// Transaction body.
    pub body: TxsTransactionBody,
    /// Transaction auth information.
    pub auth_info: TxsTransactionAuthInfo,
    /// Array of Base 64 encoded transaction signatures.
    pub signatures: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransactionBody {
    /// Transaction messages.
    pub messages: Vec<TxsTransactionMessage>,
    /// Transaction memo. Eg: `"1891420480"`
    pub memo: String,
    /// Transaction timeout height. Eg: `"0"`
    pub timeout_height: String,
    /// Array of transaction extension options.
    pub extension_options: Vec<ExtensionOption>,
    /// Non-critical transaction extension options.
    pub non_critical_extension_options: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "@type")]
pub enum ExtensionOption {
    #[serde(rename = "/ethermint.types.v1.ExtensionOptionsWeb3Tx")]
    Web3Tx {
        /// Typed data chain ID. Eg: `"9001"`
        typed_data_chain_id: String,
        /// Fee payer address. Eg: `"evmos1yrnd07xnhghehqt26kqq6glmewvtslpcxqksr9"`
        fee_payer: String,
        /// Base 64 encoded fee payer signature. Eg: `"zL6Qw0xj/dflTsV893EEtJxygv+h9NBsYOiulRWLkuoMZoamDcmLLbtpyjPtkmz6j2C8CFtCRv83tUUc+qaQTRw="`
        fee_payer_sig: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransactionAuthInfo {
    /// Transaction signer informations.
    pub signer_infos: Vec<TxsTransactionSignerInfo>,
    /// Transaction fee.
    pub fee: TxsTransactionAuthInfoFee,
}

#[derive(Deserialize, Serialize, Debug)]
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
    #[serde(rename = "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward")]
    MsgWithdrawDelegatorReward {
        /// Delegator address. Eg: `"evmos1wl8penajxqyqarw94q00cd46nvwuduq40er8sj"`
        delegator_address: String,
        /// Validator address. Eg: `"evmosvaloper1d74wdckw5vyn6gwqt4r0ruemp9n8vmwtudw848"`
        validator_address: String,
    },
    #[serde(rename = "/cosmos.gov.v1beta1.MsgVote")]
    MsgVote {
        /// Proposal ID. Eg: `"78"`
        proposal_id: String,
        /// Voter address. Eg: `"evmos16arqk5g5zntx00czgqtwjjy7dz4ex3v8fuw0t2"`
        voter: String,
        /// Vote option. Eg: `"VOTE_OPTION_YES"`
        option: String,
    },
    #[serde(rename = "/ibc.applications.transfer.v1.MsgTransfer")]
    MsgTransfer {
        // Transfer source port. Eg: `"transfer"`
        source_port: String,
        /// Transfer source channel. Eg: `"channel-0"`
        source_channel: String,
        /// Transferred token's denom and amount.
        token: DenomAmount,
        /// Sender address. Eg: `"evmos1sty49epzsz0nm53snsmy6fnl2tlvnxvdwaryxh"`
        sender: String,
        /// Receiver address. Eg: `"osmo122hxr078fjenacm08sshe4qkyekyl70ls5menu"`
        receiver: String,
        /// Timeout height.
        timeout_height: TimeoutHeight,
        /// Timeout timestamp. Eg: `"1667836893000000000"`
        timeout_timestamp: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TimeoutHeight {
    /// Timeout revision number. Eg: `"1"`
    pub revision_number: String,
    /// Timout revision height. Eg: `"6789255"`
    pub revision_height: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransactionAuthInfoFee {
    /// Amount.
    pub amount: Vec<DenomAmount>,
    /// Transaction gas limit.
    pub gas_limit: String,
    /// Transaction payer. Eg: `""`
    pub payer: String,
    /// Transaction granter. Eg: `""`
    pub granter: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransactionSignerInfo {
    pub public_key: PublicKey,
    pub mode_info: TxsTransactionModeInfo,
    /// Transaction signer info sequence. Eg: `"1"`
    pub sequence: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransactionModeInfo {
    pub single: TxsTransactionModeInfoSingle,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransactionModeInfoSingle {
    /// Mode. Eg: `"SIGN_MODE_LEGACY_AMINO_JSON"`
    pub mode: String,
}

#[derive(Deserialize, Serialize, Debug)]
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
    pub events: Vec<TxsResponseEvent<UnparsedTxEventAttribute>>,
}

#[derive(Deserialize, Serialize, Debug)]
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
#[derive(Deserialize, Serialize, Debug)]
pub struct Tx {
    // Tx body.
    pub body: TxsTransactionBody,
    // Tx auth info.
    pub auth_info: TxsTransactionAuthInfo,
    /// Array of Base 64 encoded signatures.
    pub signatures: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsResponseLog {
    /// Message index. Eg: `0`
    pub msg_index: usize,
    /// Log. Eg: `""`
    pub log: String,
    /// Events.
    pub events: Vec<TxsResponseEvent<TxResponseLogEventAttribute>>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum TxsResponseEvent<T> {
    CoinReceived {
        /// Coin received attributes.
        attributes: Vec<T>,
    },
    ProposalVote {
        /// Proposal attributes.
        attributes: Vec<T>,
    },
    CoinSpent {
        /// Coin spent attributes.
        attributes: Vec<T>,
    },
    IbcTransfer {
        /// Coin spent attributes.
        attributes: Vec<T>,
    },
    SendPacket {
        /// Send packet attributes.
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
    WithdrawRewards {
        /// Withdraw rewards attributes.
        attributes: Vec<T>,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnparsedTxEventAttribute {
    /// Unparsed event attribute key. Eg: `"cmVjaXBpZW50"`
    pub key: String,
    /// Unparsed event attribute key. Might be `None`. Eg: `"ZXZtb3MxN3hwZnZha20yYW1nOTYyeWxzNmY4NHoza2VsbDhjNWxqY2p3MzQ"`
    pub value: Option<String>,
    /// Unparsed event attribute index. Might be `None`. Eg: `true`
    pub index: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "key")]
#[serde(rename_all = "snake_case")]
pub enum TxResponseLogEventAttribute {
    Receiver {
        /// Receiver address. Eg: `"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef"`
        #[serde(rename = "value")]
        receiver_address: String,
    },
    Option {
        /// JSON encoded option object. Eg: `"{\"option\":1,\"weight\":\"1.000000000000000000\"}"`
        #[serde(rename = "value")]
        option_object: String,
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
    ProposalId {
        /// Proposal ID. Eg: `"78"`
        #[serde(rename = "value")]
        proposal_id: String,
    },
    PacketData {
        /// JSON encoded packet data. Eg: `"{\"amount\":\"10000000000000000000\",\"denom\":\"aevmos\",\"receiver\":\"osmo122hxr078fjenacm08sshe4qkyekyl70ls5menu\",\"sender\":\"evmos1sty49epzsz0nm53snsmy6fnl2tlvnxvdwaryxh\"}"`
        #[serde(rename = "value")]
        packet_data: String,
    },

    PacketDataHex {
        /// HEX encoded packet data. Eg: `"7b22616d6f756e74223a223130303030303030303030303030303030303030222c2264656e6f6d223a226165766d6f73222c227265636569766572223a226f736d6f313232687872303738666a656e61636d30387373686534716b79656b796c37306c73356d656e75222c2273656e646572223a2265766d6f7331737479343965707a737a306e6d3533736e736d7936666e6c32746c766e787664776172797868227d"`
        #[serde(rename = "value")]
        packet_data_hex: String,
    },
    PacketTimeoutHeight {
        /// Packet timeout height. Eg: `"1-6789255"`
        #[serde(rename = "value")]
        packet_timeout_height: String,
    },
    PacketTimeoutTimestamp {
        /// Packet timeout timestamp. Eg: `"1667836893000000000"`
        #[serde(rename = "value")]
        packet_timeout_timestamp: String,
    },
    PacketSequence {
        /// Packet sequence. Eg: `"868847"`
        #[serde(rename = "value")]
        packet_sequence: String,
    },
    PacketSrcPort {
        /// Packet source port. Eg: `"transfer"`
        #[serde(rename = "value")]
        packet_src_port: String,
    },
    PacketSrcChannel {
        /// Packet source channel. Eg: `"channel-0"`
        #[serde(rename = "value")]
        packet_src_channel: String,
    },
    PacketDstPort {
        /// Packet destination port. Eg: `"transfer"`
        #[serde(rename = "value")]
        packet_dst_port: String,
    },
    PacketDstChannel {
        /// Packet source channel. Eg: `"channel-204"`
        #[serde(rename = "value")]
        packet_dst_channel: String,
    },
    PacketChannelOrdering {
        /// Packet channel ordering. Eg: `"ORDER_UNORDERED"`
        #[serde(rename = "value")]
        packet_channel_ordering: String,
    },
    PacketConnection {
        /// Packet channel ordering. Eg: `"channel-0"`
        #[serde(rename = "value")]
        packet_connection: String,
    },
    Validator {
        /// Validator address. Eg: `"evmosvaloper1ce4vh0e5kanlgc7z0rhcemvd8erjnfzcyfecl7"`
        #[serde(rename = "value")]
        validator_address: String,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct TxResp {
    pub tx: Tx,
    pub tx_response: TxResponse,
}
