use actix_web::{
    get,
    web::{Data, Path},
    Responder,
};

use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};
use crate::state::State;

// ======== 'axelar' Delegator Methods ========

#[get("{chain}/delegator-rewards/{address}")]
pub async fn delegator_rewards(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, delegator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_delegator_rewards(&delegator_addr).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/delegator-withdraw-address/{address}")]
pub async fn delegator_withdraw_address(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, delegator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_delegator_withdraw_address(&delegator_addr).await?;
    Ok(TNRAppSuccessResponse::new(data))
}
