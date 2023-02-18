use actix_web::{
    get,
    Responder,
    web::{Data, Path},
};
use serde::Serialize;

use crate::state::State;
use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};

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
