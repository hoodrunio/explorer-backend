use actix_web::{
    get,
    web::{Data, Path},
    Responder,
};
use serde::Serialize;

use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};
use crate::state::State;

// ======== Staking Pool Methods ========

#[get("{chain}/staking-pool")]
pub async fn staking_pool(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    // Database can be used.
    let data = chain.get_staking_pool().await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[derive(Serialize)]
pub struct StakingPool {
    pub bonded: u64,
    pub unbonded: u64,
}
