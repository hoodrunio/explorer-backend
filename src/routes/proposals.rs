use actix_web::{
    get,
    Responder,
    web::{Data, Path, Query},
};

use crate::{
    fetch::others::PaginationConfig,
    state::State,
};
use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};

use super::QueryParams;

// ======== 'axelar' Propsals Methods ========

#[get("{chain}/proposals-passed")]
pub async fn proposals_passed(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1)).reverse();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_passed(config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/proposals-voting")]
pub async fn proposals_voting(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1)).reverse();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_in_voting_period(config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/proposals-failed")]
pub async fn proposals_failed(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1)).reverse();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_failed(config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/proposals-rejected")]
pub async fn proposals_rejected(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1)).reverse();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_rejected(config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/proposals-unspecified")]
pub async fn proposals_unspecified(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1)).reverse();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_unspecified(config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/proposal-deposits/{id}")]
pub async fn proposal_deposits(path: Path<(String, u64)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_deposits(proposal_id, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/proposal-details/{id}")]
pub async fn proposal_details(path: Path<(String, u64)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_details(proposal_id).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/proposal-tally/{id}")]
pub async fn proposal_tally(path: Path<(String, u64)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_tally(proposal_id).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/proposal-votes/{id}")]
pub async fn proposal_votes(path: Path<(String, u64)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_votes(proposal_id, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote(path: Path<(String, u64, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id, voter_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_vote_by_voter(proposal_id, &voter_addr).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit(path: Path<(String, u64, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id, depositor_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_deposit_by_depositor(proposal_id, &depositor_addr).await?;
    Ok(TNRAppSuccessResponse::new(data))
}
