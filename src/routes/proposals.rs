use actix_web::{
    get,
    web::{Data, Path, Query},
    Responder,
};

use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};
use crate::{fetch::others::PaginationConfig, state::State};
use crate::{fetch::socket::EvmPollVote::No, routes::PaginationData};

use super::QueryParams;

// ======== 'axelar' Propsals Methods ========

#[get("{chain}/proposals-passed")]
pub async fn proposals_passed(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_passed(query.0).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposals-voting")]
pub async fn proposals_voting(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_in_voting_period(query.0).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposals-failed")]
pub async fn proposals_failed(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_failed(query.0).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposals-rejected")]
pub async fn proposals_rejected(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_rejected(query.0).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposals-unspecified")]
pub async fn proposals_unspecified(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_unspecified(query.0).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposal-deposits/{id}")]
pub async fn proposal_deposits(
    path: Path<(String, u64)>,
    chains: Data<State>,
    query: Query<PaginationConfig>,
) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.0.page);

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_deposits(proposal_id, config).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposal-details/{id}")]
pub async fn proposal_details(path: Path<(String, u64)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_details(proposal_id).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposal-tally/{id}")]
pub async fn proposal_tally(path: Path<(String, u64)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_tally(proposal_id).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposal-votes/{id}")]
pub async fn proposal_votes(path: Path<(String, u64)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_votes(proposal_id, config).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote(path: Path<(String, u64, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id, voter_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_vote_by_voter(proposal_id, &voter_addr).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit(path: Path<(String, u64, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id, depositor_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_deposit_by_depositor(proposal_id, &depositor_addr).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}
