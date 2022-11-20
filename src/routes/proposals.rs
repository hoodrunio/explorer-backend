use super::QueryParams;
use crate::{
    fetch::others::{PaginationConfig, Response},
    state::State,
};
use actix_web::{
    get,
    web::{Data, Json, Path, Query},
    Responder,
};

// ======== 'axelar' Propsals Methods ========

#[get("{chain}/proposals-passed")]
pub async fn proposals_passed(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1)).reverse();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposals_passed(config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/proposals-voting")]
pub async fn proposals_voting(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1)).reverse();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposals_in_voting_period(config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/proposals-failed")]
pub async fn proposals_failed(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1)).reverse();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposals_failed(config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/proposals-rejected")]
pub async fn proposals_rejected(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1)).reverse();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposals_rejected(config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/proposals-unspecified")]
pub async fn proposals_unspecified(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1)).reverse();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposals_unspecified(config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/proposal-deposits/{id}")]
pub async fn proposal_deposits(path: Path<(String, u64)>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let (chain, proposal_id) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1));

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposal_deposits(proposal_id, config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/proposal-details/{id}")]
pub async fn proposal_details(path: Path<(String, u64)>, chains: Data<State>) -> impl Responder {
    let (chain, proposal_id) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposal_details(proposal_id).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/proposal-tally/{id}")]
pub async fn proposal_tally(path: Path<(String, u64)>, chains: Data<State>) -> impl Responder {
    let (chain, proposal_id) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposal_tally(proposal_id).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/proposal-votes/{id}")]
pub async fn proposal_votes(path: Path<(String, u64)>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let (chain, proposal_id) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1));

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposal_votes(proposal_id, config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote(path: Path<(String, u64, String)>, chains: Data<State>) -> impl Responder {
    let (chain, proposal_id, voter_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposal_vote_by_voter(proposal_id, &voter_addr).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit(path: Path<(String, u64, String)>, chains: Data<State>) -> impl Responder {
    let (chain, proposal_id, depositor_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_proposal_deposit_by_depositor(proposal_id, &depositor_addr).await.into(),
        Err(err) => Response::Error(err),
    })
}
