use actix_web::{
    get,
    Responder,
    web::{Data, Json, Path},
};
use actix_web::web::Query;
use serde::{Deserialize, Serialize};

use crate::{fetch::others::Response, state::State};
use crate::chain::Chain;
use crate::database::{HeartbeatForDb, ListDbResult, PaginationDb};
use crate::fetch::heartbeats::{HeartbeatsListElement, HeartbeatsQuery};
use crate::fetch::others::PaginationConfig;
use crate::routes::{extract_chain, QueryParams, TNRAppError, TNRAppSuccessResponse};

// ====== Heart Beats Methods ======

#[get("{chain}/validator/heartbeats/{operator_address}")]
pub async fn validator_hearbeats(path: Path<(String, String)>, chains: Data<State>, body: Option<Json<ValidatorHeartbeatsQBody>>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, operator_address) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(String::from(format!("Evm polls not supported for {}", &chain.config.name))));
    };
    let config = PaginationConfig::new().limit(query.limit.unwrap_or(250)).page(query.page.unwrap_or(1));

    let (from, to) = match body {
        None => { (None, None) }
        Some(res) => {
            let body_inner = res.into_inner();
            (body_inner.from_block, body_inner.to_block)
        }
    };
    let heartbeats_query = HeartbeatsQuery::new(
        from.clone(),
        to.clone())?;

    let data = chain.get_val_heartbeats(operator_address, heartbeats_query, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidatorHeartbeatsQBody {
    pub from_block: Option<i64>,
    pub sender: String,
    pub to_block: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HeartbeatsListResp {
    pub list: Vec<HeartbeatsListElement>,
    pub pagination: PaginationDb,
}


impl HeartbeatsListResp {
    pub fn from_db_list(list_db_result: ListDbResult<HeartbeatForDb>) -> Result<Self, TNRAppError> {
        let heartbeats = list_db_result.list.into_iter().map(|heartbeat| {
            HeartbeatsListElement {
                tx_hash: heartbeat.tx_hash.clone(),
                height: heartbeat.height.clone(),
                period_height: heartbeat.period_height.clone(),
                timestamp: heartbeat.timestamp.clone(),
                signatures: heartbeat.signatures.clone(),
                sender: heartbeat.sender.clone(),
                key_ids: heartbeat.key_ids.clone(),
                id: heartbeat.id.clone(),
            }
        }).collect();

        Ok(Self {
            list: heartbeats,
            pagination: list_db_result.pagination,
        })
    }
}