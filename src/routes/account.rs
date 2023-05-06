use crate::{fetch::others::PaginationConfig, routes::QueryParams, state::State};
use actix_web::{
    get,
    web::{Data, Path, Query},
    Responder,
};

use crate::routes::{extract_chain, PaginationData, TNRAppError, TNRAppSuccessResponse};

// ======== Account Methods ========

//account address => evmos198zkgedxs9f77ru80zd3g693dhpv6n5wej6d7p
#[get("{chain}/account/{account_address}")]
pub async fn account(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, account_address) = path.into_inner();
    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_account_info(&account_address).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/vesting/{account_address}")]
pub async fn account_vesting(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, account_address) = path.into_inner();
    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_account_vesting_info(account_address).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/balances/{account_address}")]
pub async fn account_balances(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, account_address) = path.into_inner();
    let config = PaginationData {
        cursor: None,
        offset: None,
        limit: Some(1000),
        direction: None,
    };
    let chain = extract_chain(&chain, chains)?;

    let data = chain.get_account_balances(&account_address, config).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}
