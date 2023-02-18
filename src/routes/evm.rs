use actix_web::{
    get,
    Responder,
    web::{Data, Path},
};
use actix_web::web::Query;
use mongodb::bson::doc;
use mongodb::options::FindOptions;

use crate::state::State;
use crate::fetch::evm::{EvmPollListResp, EvmVotesListResp};
use crate::fetch::others::PaginationConfig;
use crate::routes::{create_options, extract_chain, PaginationData, QueryParams, TNRAppError, TNRAppSuccessResponse};

// ====== Evm Methods ======

#[get("{chain}/evm/polls")]
pub async fn evm_polls(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(String::from(format!("Evm polls not supported for {}", &chain.config.name))));
    };

    let evm_polls_from_db = chain.database.find_paginated_evm_polls(None, Some(query.0)).await?;
    Ok(evm_polls_from_db)
}

#[get("{chain}/evm/poll/{poll_id}")]
pub async fn evm_poll(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, poll_id) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(String::from(format!("Evm polls not supported for {}", &chain.config.name))));
    };

    let data = chain.get_evm_poll(&poll_id).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/evm/votes/{operator_address}")]
pub async fn evm_validator_votes(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, operator_address) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(String::from(format!("Evm votes not supported for {}", &chain.config.name))));
    };

    let config = PaginationConfig::new().limit(query.limit.unwrap_or(20)).page(query.page.unwrap_or(1));

    let val_evm_polls_from_db = chain.database.find_paginated_evm_polls(Some(doc! {"$match":{"participants":{"$elemMatch":{"operator_address":operator_address.clone()}}}}), None).await?;
    Ok(val_evm_polls_from_db)
}

#[get("{chain}/evm/validator/supported_chains/{operator_address}")]
pub async fn evm_val_supported_chains(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, operator_address) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(String::from(format!("Evm polls not supported for {}", &chain.config.name))));
    };

    let data: Vec<String> = chain.get_supported_chains(&operator_address).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}
