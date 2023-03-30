use actix_web::web::Query;
use actix_web::{
    get,
    web::{Data, Path},
    Responder,
};
use mongodb::bson::doc;

use crate::routes::{extract_chain, PaginationData, TNRAppError, TNRAppSuccessResponse};
use crate::state::State;

// ====== Evm Methods ======

#[get("{chain}/evm/polls")]
pub async fn evm_polls(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(format!("Evm polls not supported for {}", &chain.config.name)));
    };

    let data = chain.get_evm_polls(None, query.into_inner()).await?;
    Ok(TNRAppSuccessResponse::from(data))
}

#[get("{chain}/evm/poll/{poll_id}")]
pub async fn evm_poll(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, poll_id) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(format!("Evm polls not supported for {}", &chain.config.name)));
    };

    let data = chain.get_evm_poll(&poll_id).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/evm/votes/{operator_address}")]
pub async fn evm_validator_votes(
    path: Path<(String, String)>,
    chains: Data<State>,
    query: Query<PaginationData>,
) -> Result<impl Responder, TNRAppError> {
    let (chain, operator_address) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(format!("Evm votes not supported for {}", &chain.config.name)));
    };

    let data = chain
        .database
        .find_paginated_evm_poll_participants(Some(doc! {"operator_address":operator_address.clone()}), query.into_inner())
        .await?;

    Ok(TNRAppSuccessResponse::from(data))
}

#[get("{chain}/evm/validator/supported_chains/{operator_address}")]
pub async fn evm_val_supported_chains(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, operator_address) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;

    if &chain.config.name != "axelar" {
        return Err(TNRAppError::from(format!("Evm polls not supported for {}", &chain.config.name)));
    };

    let data: Vec<String> = chain.get_supported_chains(&operator_address).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}
