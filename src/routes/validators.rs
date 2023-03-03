use actix_web::{
    get,
    Responder,
    web::{Data, Path, Query},
};
use mongodb::bson::doc;
use serde::Deserialize;

use crate::{
    fetch::others::PaginationConfig,
    state::State,
};
use crate::fetch::validators::{ValidatorListResp, ValidatorRedelegationQuery};
use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};

use super::QueryParams;

// ======== Validator Methods ========

#[get("{chain}/validator/{address}")]
pub async fn validator(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_info(&validator_addr).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validator-delegations/{address}")]
pub async fn validator_delegations(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_delegations(&validator_addr, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validator-unbondings/{address}")]
pub async fn validator_unbondings(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_unbondings(&validator_addr, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validator-redelegations/{address}")]
pub async fn validator_redelegations(path: Path<(String, String)>, chains: Data<State>, query: Query<ValidatorRedelegationQueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(query.limit.unwrap_or(10)).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let query_config = ValidatorRedelegationQuery { source: query.source, destination: query.destination };
    let data = chain.get_validator_redelegations(&validator_addr, config, query_config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validator-commission/{address}")]
pub async fn validator_commission(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_commission(&validator_addr).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validator-rewards/{address}")]
pub async fn validator_rewards(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_rewards(&validator_addr).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validators-bonded")]
pub async fn validators_bonded(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(query.limit.unwrap_or(20)).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let validator_db_resp = chain.database.find_paginated_validators(Some(doc! {"$match":{"is_active":true}}), config).await?;
    let data = ValidatorListResp::from_db_list(validator_db_resp, &chain).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validators-unbonded")]
pub async fn validators_unbonded(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(query.limit.unwrap_or(20)).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let validator_db_resp = chain.database.find_paginated_validators(Some(doc! {"$match":{"is_active":false}}), config).await?;
    let data = ValidatorListResp::from_db_list(validator_db_resp, &chain).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validators-unbonding")]
pub async fn validators_unbonding(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validators_unbonding(PaginationConfig::new()).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validators-unspecified")]
pub async fn validators_unspecified(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validators_unspecified(PaginationConfig::new()).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validators-of/{address}")]
pub async fn validators_of_delegator(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, delegator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validators_by_delegator(&delegator_addr, PaginationConfig::new()).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair(path: Path<(String, String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr, delegator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_delegator_validator_pair_info(&delegator_addr, &validator_addr).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validator-set/{height}")]
pub async fn validator_set_by_height(path: Path<(String, u64)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, height) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_set_by_height(height).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/validator-set")]
pub async fn validator_set(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_set().await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[derive(Deserialize)]
pub struct ValidatorRedelegationQueryParams {
    pub page: Option<u8>,
    pub limit: Option<u16>,
    pub source: Option<bool>,
    pub destination: Option<bool>,
}
