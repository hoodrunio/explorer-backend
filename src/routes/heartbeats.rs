use actix_web::web::Query;
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::database::{HeartbeatForDb, ListDbResult};
use crate::fetch::heartbeats::{HeartbeatsListElement, HeartbeatsListRawElement, HeartbeatsQuery};
use crate::routes::{extract_chain, PaginationData, TNRAppError, TNRAppSuccessResponse};
use crate::state::State;

// ====== Heart Beats Methods ======

#[get("{chain}/validator/heartbeats/{operator_address}")]
pub async fn validator_hearbeats(
    path: Path<(String, String)>,
    chains: Data<State>,
    body: Option<Json<ValidatorHeartbeatsQBody>>,
    query: Query<PaginationData>,
) -> Result<TNRAppSuccessResponse<Vec<HeartbeatForDb>>, TNRAppError> {
    let (chain, operator_address) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(format!("Heartbeats not supported for {}", &chain.config.name)));
    };

    let (from, to) = match body {
        None => (None, None),
        Some(res) => {
            let body_inner = res.into_inner();
            (body_inner.from_block, body_inner.to_block)
        }
    };
    let heartbeats_query = HeartbeatsQuery::new(from.clone(), to.clone())?;

    let data = chain.get_val_heartbeats(operator_address, heartbeats_query, query.into_inner()).await?;
    Ok(TNRAppSuccessResponse::new(data.data, None))
}

#[get("{chain}/heartbeats")]
pub async fn hearbeats(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(format!("Hearbeats not supported for {}", &chain.config.name)));
    };

    let data = chain
        .database
        .find_paginated_heartbeats(Some(doc! {"$match":{"sender": {"$exists":true }}}), Some(query.into_inner()))
        .await?;
    Ok(TNRAppSuccessResponse::new(data.data, Some(data.pagination)))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidatorHeartbeatsQBody {
    pub from_block: Option<i64>,
    pub to_block: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HeartbeatsListResp {
    pub list: Vec<HeartbeatsListElement>,
    pub pagination: PaginationData,
}
