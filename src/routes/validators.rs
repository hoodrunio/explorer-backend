use actix_web::{
    get,
    web::{Data, Path, Query},
    Responder,
};
use mongodb::bson::doc;
use serde::Deserialize;
use tendermint::evidence::List;

use crate::{
    fetch::validators::InternalRedelegation,
    routes::{extract_chain, PaginationData, TNRAppError, TNRAppSuccessResponse},
};
use crate::{
    fetch::validators::{ValidatorListResp, ValidatorRedelegationQuery},
    state::State,
};
use crate::database::ListDbResult;

// ======== Validator Methods ========

#[get("{chain}/validator/{address}")]
pub async fn validator(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_info(&validator_addr).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/validator-delegations/{address}")]
pub async fn validator_delegations(
    path: Path<(String, String)>,
    chains: Data<State>,
    query: Query<PaginationData>,
) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_delegations(&validator_addr, query.into_inner()).await?;
    Ok(TNRAppSuccessResponse::from(data))
}

#[get("{chain}/validator-unbondings/{address}")]
pub async fn validator_unbondings(
    path: Path<(String, String)>,
    chains: Data<State>,
    query: Query<PaginationData>,
) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_unbondings(&validator_addr, query.into_inner()).await?;
    Ok(TNRAppSuccessResponse::from(data))
}

#[get("{chain}/validator-redelegations/{address}")]
pub async fn validator_redelegations(
    path: Path<(String, String)>,
    chains: Data<State>,
    query: Query<ValidatorRedelegationQueryParams>,
) -> Result<TNRAppSuccessResponse<Vec<InternalRedelegation>>, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let query_config = ValidatorRedelegationQuery {
        source: query.source,
        destination: query.destination,
    };
    let data = chain
        .get_validator_redelegations(&validator_addr, query.pagination.clone(), query_config)
        .await?;
    Ok(data.into())
}

#[get("{chain}/validator-commission/{address}")]
pub async fn validator_commission(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_commission(&validator_addr).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/validator-rewards/{address}")]
pub async fn validator_rewards(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_rewards(&validator_addr).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/validators-bonded")]
pub async fn validators_bonded(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();
    let chain = extract_chain(&chain, chains)?;
    let validator_db_resp = chain
        .database
        .find_paginated_validators(Some(doc! { "is_active": true }), query.into_inner())
        .await?;

    let pagination = validator_db_resp.pagination.clone();
    let data = ValidatorListResp::from_db_list(validator_db_resp, &chain).await?;

    Ok(TNRAppSuccessResponse::new(data, Some(pagination)))
}

#[get("{chain}/validators-unbonded")]
pub async fn validators_unbonded(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();
    let chain = extract_chain(&chain, chains)?;
    let validator_db_resp = chain
        .database
        .find_paginated_validators(Some(doc! {"is_active":false}), query.into_inner())
        .await?;

    let pagination = validator_db_resp.pagination.clone();
    let data = ValidatorListResp::from_db_list(validator_db_resp, &chain).await?;

    Ok(TNRAppSuccessResponse::new(data, Some(pagination)))
}

#[get("{chain}/validators-unbonding")]
pub async fn validators_unbonding(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validators_unbonding(PaginationData::default()).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/validators-unspecified")]
pub async fn validators_unspecified(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validators_unspecified(PaginationData::default()).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/validators-of/{address}")]
pub async fn validators_of_delegator(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, delegator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validators_by_delegator(&delegator_addr, PaginationData::default()).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair(path: Path<(String, String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, validator_addr, delegator_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_delegator_validator_pair_info(&delegator_addr, &validator_addr).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/validator-set/{height}")]
pub async fn validator_set_by_height(path: Path<(String, i64)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, height) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_set_by_height(height).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/validator-set")]
pub async fn validator_set(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_set().await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[derive(Deserialize)]
pub struct ValidatorRedelegationQueryParams {
    #[serde(flatten)]
    pub pagination: PaginationData,
    pub source: Option<bool>,
    pub destination: Option<bool>,
}
