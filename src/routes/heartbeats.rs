use actix_web::{
    get,
    Responder,
    web::{Data, Json, Path},
};
use actix_web::web::Query;
use serde::{Deserialize, Serialize};

use crate::{fetch::others::Response, state::State};
use crate::chain::Chain;
use crate::routes::{extract_chain, QueryParams, TNRAppError, TNRAppSuccessResponse};

// ====== Heart Beats Methods ======

#[get("{chain}/validator/heartbeats/{operator_address}")]
pub async fn validator_hearbeats(path: Path<(String, String)>, chains: Data<State>, body: Json<ValidatorHeartbeatsQBody>) -> Result<impl Responder, TNRAppError> {
    let (chain, operator_address) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(String::from(format!("Evm polls not supported for {}", &chain.config.name))));
    };

    let data = chain.get_val_heartbeats(operator_address, body.into_inner()).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidatorHeartbeatsQBody {
    pub from_block: Option<u64>,
    // pub size:u64,
    pub sender: String,
    pub to_block: Option<u64>,
}