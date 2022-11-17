use serde::{Deserialize, Serialize};
use strum_macros::IntoStaticStr;

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
pub struct Event {
    pub r#type: String,
    pub attributes: Vec<UnparsedEventAttribute>,
}
