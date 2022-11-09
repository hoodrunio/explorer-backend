use super::necessities::*;

// ==== AXELAR ====

#[get("axelar/proposals-passed")]
pub async fn proposals_passed_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_proposals_passed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/proposals-voting")]
pub async fn proposals_voting_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_proposals_in_voting_period(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/proposals-failed")]
pub async fn proposals_failed_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_proposals_failed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/proposals-rejected")]
pub async fn proposals_rejected_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_proposals_rejected(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/proposals-unspecified")]
pub async fn proposals_unspecified_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_proposals_unspecified(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/proposal-deposits/{id}")]
pub async fn proposal_deposits_axelar(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_proposal_deposits(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/proposal-details/{id}")]
pub async fn proposal_details_axelar(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_proposal_details(id)
        .await;

    Json(resp)
}

#[get("axelar/proposal-tally/{id}")]
pub async fn proposal_tally_axelar(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_proposal_tally(id)
        .await;

    Json(resp)
}

#[get("axelar/proposal-votes/{id}")]
pub async fn proposal_votes_axelar(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_proposal_votes(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote_axelar(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, voter_address) = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_proposal_vote_by_voter(id, &voter_address).await;

    Json(resp)
}

#[get("axelar/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit_axelar(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, depositor_address) = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_proposal_deposit_by_depositor(id, &depositor_address).await;

    Json(resp)
}

// ==== CELESTIA ====

#[get("celestia/proposals-passed")]
pub async fn proposals_passed_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_proposals_passed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/proposals-voting")]
pub async fn proposals_voting_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_proposals_in_voting_period(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/proposals-failed")]
pub async fn proposals_failed_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_proposals_failed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/proposals-rejected")]
pub async fn proposals_rejected_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_proposals_rejected(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/proposals-unspecified")]
pub async fn proposals_unspecified_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_proposals_unspecified(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/proposal-deposits/{id}")]
pub async fn proposal_deposits_celestia(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_proposal_deposits(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/proposal-details/{id}")]
pub async fn proposal_details_celestia(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_proposal_details(id)
        .await;

    Json(resp)
}

#[get("celestia/proposal-tally/{id}")]
pub async fn proposal_tally_celestia(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_proposal_tally(id)
        .await;

    Json(resp)
}

#[get("celestia/proposal-votes/{id}")]
pub async fn proposal_votes_celestia(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_proposal_votes(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote_celestia(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, voter_address) = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_proposal_vote_by_voter(id, &voter_address).await;

    Json(resp)
}

#[get("celestia/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit_celestia(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, depositor_address) = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_proposal_deposit_by_depositor(id, &depositor_address).await;

    Json(resp)
}

// ==== COSMOS ====

#[get("cosmos/proposals-passed")]
pub async fn proposals_passed_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_proposals_passed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/proposals-voting")]
pub async fn proposals_voting_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_proposals_in_voting_period(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/proposals-failed")]
pub async fn proposals_failed_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_proposals_failed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/proposals-rejected")]
pub async fn proposals_rejected_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_proposals_rejected(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/proposals-unspecified")]
pub async fn proposals_unspecified_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_proposals_unspecified(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/proposal-deposits/{id}")]
pub async fn proposal_deposits_cosmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_proposal_deposits(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/proposal-details/{id}")]
pub async fn proposal_details_cosmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_proposal_details(id)
        .await;

    Json(resp)
}

#[get("cosmos/proposal-tally/{id}")]
pub async fn proposal_tally_cosmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_proposal_tally(id)
        .await;

    Json(resp)
}

#[get("cosmos/proposal-votes/{id}")]
pub async fn proposal_votes_cosmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_proposal_votes(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote_cosmos(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, voter_address) = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_proposal_vote_by_voter(id, &voter_address).await;

    Json(resp)
}

#[get("cosmos/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit_cosmos(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, depositor_address) = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_proposal_deposit_by_depositor(id, &depositor_address).await;

    Json(resp)
}

// ==== EVMOS ====

#[get("evmos/proposals-passed")]
pub async fn proposals_passed_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_proposals_passed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/proposals-voting")]
pub async fn proposals_voting_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_proposals_in_voting_period(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/proposals-failed")]
pub async fn proposals_failed_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_proposals_failed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/proposals-rejected")]
pub async fn proposals_rejected_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_proposals_rejected(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/proposals-unspecified")]
pub async fn proposals_unspecified_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_proposals_unspecified(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/proposal-deposits/{id}")]
pub async fn proposal_deposits_evmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_proposal_deposits(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/proposal-details/{id}")]
pub async fn proposal_details_evmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_proposal_details(id)
        .await;

    Json(resp)
}

#[get("evmos/proposal-tally/{id}")]
pub async fn proposal_tally_evmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_proposal_tally(id)
        .await;

    Json(resp)
}

#[get("evmos/proposal-votes/{id}")]
pub async fn proposal_votes_evmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_proposal_votes(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote_evmos(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, voter_address) = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_proposal_vote_by_voter(id, &voter_address).await;

    Json(resp)
}

#[get("evmos/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit_evmos(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, depositor_address) = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_proposal_deposit_by_depositor(id, &depositor_address).await;

    Json(resp)
}

// ==== KYVE ====

#[get("kyve/proposals-passed")]
pub async fn proposals_passed_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_proposals_passed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/proposals-voting")]
pub async fn proposals_voting_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_proposals_in_voting_period(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/proposals-failed")]
pub async fn proposals_failed_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_proposals_failed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/proposals-rejected")]
pub async fn proposals_rejected_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_proposals_rejected(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/proposals-unspecified")]
pub async fn proposals_unspecified_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_proposals_unspecified(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/proposal-deposits/{id}")]
pub async fn proposal_deposits_kyve(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_proposal_deposits(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/proposal-details/{id}")]
pub async fn proposal_details_kyve(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_proposal_details(id)
        .await;

    Json(resp)
}

#[get("kyve/proposal-tally/{id}")]
pub async fn proposal_tally_kyve(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_proposal_tally(id)
        .await;

    Json(resp)
}

#[get("kyve/proposal-votes/{id}")]
pub async fn proposal_votes_kyve(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_proposal_votes(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote_kyve(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, voter_address) = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_proposal_vote_by_voter(id, &voter_address).await;

    Json(resp)
}

#[get("kyve/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit_kyve(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, depositor_address) = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_proposal_deposit_by_depositor(id, &depositor_address).await;

    Json(resp)
}

// ==== OSMOSIS ====

#[get("osmosis/proposals-passed")]
pub async fn proposals_passed_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_proposals_passed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/proposals-voting")]
pub async fn proposals_voting_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_proposals_in_voting_period(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/proposals-failed")]
pub async fn proposals_failed_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_proposals_failed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/proposals-rejected")]
pub async fn proposals_rejected_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_proposals_rejected(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/proposals-unspecified")]
pub async fn proposals_unspecified_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_proposals_unspecified(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/proposal-deposits/{id}")]
pub async fn proposal_deposits_osmosis(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_proposal_deposits(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/proposal-details/{id}")]
pub async fn proposal_details_osmosis(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_proposal_details(id)
        .await;

    Json(resp)
}

#[get("osmosis/proposal-tally/{id}")]
pub async fn proposal_tally_osmosis(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_proposal_tally(id)
        .await;

    Json(resp)
}

#[get("osmosis/proposal-votes/{id}")]
pub async fn proposal_votes_osmosis(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_proposal_votes(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote_osmosis(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, voter_address) = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_proposal_vote_by_voter(id, &voter_address).await;

    Json(resp)
}

#[get("osmosis/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit_osmosis(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, depositor_address) = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_proposal_deposit_by_depositor(id, &depositor_address).await;

    Json(resp)
}

// ==== SECRET ====

#[get("secret/proposals-passed")]
pub async fn proposals_passed_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_proposals_passed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/proposals-voting")]
pub async fn proposals_voting_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_proposals_in_voting_period(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/proposals-failed")]
pub async fn proposals_failed_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_proposals_failed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/proposals-rejected")]
pub async fn proposals_rejected_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_proposals_rejected(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/proposals-unspecified")]
pub async fn proposals_unspecified_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_proposals_unspecified(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/proposal-deposits/{id}")]
pub async fn proposal_deposits_secret(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain
        .get_proposal_deposits(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/proposal-details/{id}")]
pub async fn proposal_details_secret(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain
        .get_proposal_details(id)
        .await;

    Json(resp)
}

#[get("secret/proposal-tally/{id}")]
pub async fn proposal_tally_secret(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain
        .get_proposal_tally(id)
        .await;

    Json(resp)
}

#[get("secret/proposal-votes/{id}")]
pub async fn proposal_votes_secret(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain
        .get_proposal_votes(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote_secret(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, voter_address) = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_proposal_vote_by_voter(id, &voter_address).await;

    Json(resp)
}

#[get("secret/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit_secret(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, depositor_address) = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_proposal_deposit_by_depositor(id, &depositor_address).await;

    Json(resp)
}

