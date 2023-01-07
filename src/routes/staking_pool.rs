use crate::routes::{extract_chain, OutRestResponse, TNRAppError, TNRAppSuccessResponse};
use crate::{fetch::others::Response, state::State};
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};
use serde::Serialize;

// ======== Staking Pool Methods ========

#[get("{chain}/staking-pool")]
pub async fn staking_pool(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    // Database can be used.
    let data = chain.get_staking_pool().await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[derive(Serialize)]
pub struct StakingPool {
    pub bonded: u64,
    pub unbonded: u64,
}
