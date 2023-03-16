use actix_web::{
    get,
    web::{Data, Path, Query},
    Responder,
};

use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};
use crate::{fetch::others::PaginationConfig, state::State};
use crate::{routes::PaginationData};
use serde::{Serialize,Deserialize};

use super::QueryParams;

// ======== 'axelar' Propsals Methods ========

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Unspecified,
    DepositPeriod,
    VotingPeriod,
    Passed,
    Rejected,
    Failed,
}

impl ProposalStatus {
    pub fn get_id(&self) -> u8 {
        use ProposalStatus::*;
        match self {
            Unspecified => 0,
            DepositPeriod => 1,
            VotingPeriod => 2,
            Passed => 3,
            Rejected => 4,
            Failed => 5,
        }
    }

    pub fn from_id(id: i32) -> Self {
        use ProposalStatus::*;
        match id {
            0 => Unspecified,
            1 => DepositPeriod,
            2 => VotingPeriod,
            3 => Passed,
            4 => Rejected,
            5 => Failed,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalsQueryParams {
    status: Option<ProposalStatus>,
    #[serde(flatten)]
    pagination: PaginationData
}

#[get("{chain}/proposals")]
pub async fn proposals(path: Path<String>, chains: Data<State>, query: Query<ProposalsQueryParams>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposals_by_status(query.0.status.unwrap_or_else(|| ProposalStatus::Unspecified), query.0.pagination).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/proposal-deposits/{id}")]
pub async fn proposal_deposits(
    path: Path<(String, u64)>,
    chains: Data<State>,
    query: Query<PaginationData>
) -> Result<impl Responder, TNRAppError> {
    let (chain, proposal_id) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_proposal_deposits(proposal_id, query.0).await?;
    Ok(TNRAppSuccessResponse::from(data))
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
