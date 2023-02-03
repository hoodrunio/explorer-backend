use actix_web::{
    get,
    Responder,
    web::{Data, Json, Path},
};
use actix_web::web::Query;

use crate::{fetch::others::Response, state::State};
use crate::chain::Chain;
use crate::routes::{extract_chain, QueryParams, TNRAppError, TNRAppSuccessResponse};

// ====== Heart Beats Methods ======

#[get("{chain}/validator/heartbeats/{operator_address}")]
pub async fn validator_hearbeats(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, operator_address) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(String::from(format!("Evm polls not supported for {}", &chain.config.name))));
    };

    let data: String = chain.get_val_heartbeats(&operator_address).await?;
    Ok(TNRAppSuccessResponse::new(data))
}