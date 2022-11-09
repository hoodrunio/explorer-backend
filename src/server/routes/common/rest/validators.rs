use super::necessities::*;

// ==== AXELAR ====

#[get("axelar/validator/{address}")]
pub async fn validator_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_validator(&validator_address).await;

    Json(resp)
}

#[get("axelar/validator-commission/{address}")]
pub async fn validator_commission_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_validator_commission(&validator_address).await;

    Json(resp)
}

#[get("axelar/validator-rewards/{address}")]
pub async fn validator_rewards_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_validator_rewards(&validator_address).await;

    Json(resp)
}

#[get("axelar/validators-bonded")]
pub async fn validators_bonded_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_validators_bonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/validators-unbonded")]
pub async fn validators_unbonded_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_validators_unbonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/validators-unbonding")]
pub async fn validators_unbonding_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_validators_unbonding(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/validators-unspecified")]
pub async fn validators_unspecified_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_validators_unspecified(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/validators-of/{address}")]
pub async fn validators_of_delegator_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_validators_by_delegator(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair_axelar(path: Path<(String, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (validator_address, delegator_address) = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_delegator_validator_pair_info(&delegator_address, &validator_address)
        .await;

    Json(resp)
}

// ==== CELESTIA ====

#[get("celestia/validator/{address}")]
pub async fn validator_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_validator(&validator_address).await;

    Json(resp)
}

#[get("celestia/validator-commission/{address}")]
pub async fn validator_commission_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_validator_commission(&validator_address).await;

    Json(resp)
}

#[get("celestia/validator-rewards/{address}")]
pub async fn validator_rewards_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_validator_rewards(&validator_address).await;

    Json(resp)
}

#[get("celestia/validators-bonded")]
pub async fn validators_bonded_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_validators_bonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/validators-unbonded")]
pub async fn validators_unbonded_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_validators_unbonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/validators-unbonding")]
pub async fn validators_unbonding_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_validators_unbonding(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/validators-unspecified")]
pub async fn validators_unspecified_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_validators_unspecified(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/validators-of/{address}")]
pub async fn validators_of_delegator_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_validators_by_delegator(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair_celestia(path: Path<(String, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (validator_address, delegator_address) = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_delegator_validator_pair_info(&delegator_address, &validator_address)
        .await;

    Json(resp)
}

// ==== COSMOS ====

#[get("cosmos/validator/{address}")]
pub async fn validator_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_validator(&validator_address).await;

    Json(resp)
}

#[get("cosmos/validator-commission/{address}")]
pub async fn validator_commission_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_validator_commission(&validator_address).await;

    Json(resp)
}

#[get("cosmos/validator-rewards/{address}")]
pub async fn validator_rewards_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_validator_rewards(&validator_address).await;

    Json(resp)
}

#[get("cosmos/validators-bonded")]
pub async fn validators_bonded_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_validators_bonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/validators-unbonded")]
pub async fn validators_unbonded_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_validators_unbonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/validators-unbonding")]
pub async fn validators_unbonding_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_validators_unbonding(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/validators-unspecified")]
pub async fn validators_unspecified_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_validators_unspecified(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/validators-of/{address}")]
pub async fn validators_of_delegator_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_validators_by_delegator(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair_cosmos(path: Path<(String, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (validator_address, delegator_address) = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_delegator_validator_pair_info(&delegator_address, &validator_address)
        .await;

    Json(resp)
}

// ==== EVMOS ====

#[get("evmos/validator/{address}")]
pub async fn validator_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_validator(&validator_address).await;

    Json(resp)
}

#[get("evmos/validator-commission/{address}")]
pub async fn validator_commission_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_validator_commission(&validator_address).await;

    Json(resp)
}

#[get("evmos/validator-rewards/{address}")]
pub async fn validator_rewards_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_validator_rewards(&validator_address).await;

    Json(resp)
}

#[get("evmos/validators-bonded")]
pub async fn validators_bonded_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_validators_bonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/validators-unbonded")]
pub async fn validators_unbonded_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_validators_unbonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/validators-unbonding")]
pub async fn validators_unbonding_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_validators_unbonding(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/validators-unspecified")]
pub async fn validators_unspecified_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_validators_unspecified(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/validators-of/{address}")]
pub async fn validators_of_delegator_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_validators_by_delegator(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair_evmos(path: Path<(String, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (validator_address, delegator_address) = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_delegator_validator_pair_info(&delegator_address, &validator_address)
        .await;

    Json(resp)
}

// ==== KYVE ====

#[get("kyve/validator/{address}")]
pub async fn validator_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_validator(&validator_address).await;

    Json(resp)
}

#[get("kyve/validator-commission/{address}")]
pub async fn validator_commission_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_validator_commission(&validator_address).await;

    Json(resp)
}

#[get("kyve/validator-rewards/{address}")]
pub async fn validator_rewards_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_validator_rewards(&validator_address).await;

    Json(resp)
}

#[get("kyve/validators-bonded")]
pub async fn validators_bonded_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_validators_bonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/validators-unbonded")]
pub async fn validators_unbonded_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_validators_unbonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/validators-unbonding")]
pub async fn validators_unbonding_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_validators_unbonding(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/validators-unspecified")]
pub async fn validators_unspecified_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_validators_unspecified(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/validators-of/{address}")]
pub async fn validators_of_delegator_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_validators_by_delegator(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair_kyve(path: Path<(String, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (validator_address, delegator_address) = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_delegator_validator_pair_info(&delegator_address, &validator_address)
        .await;

    Json(resp)
}

// ==== OSMOSIS ====

#[get("osmosis/validator/{address}")]
pub async fn validator_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_validator(&validator_address).await;

    Json(resp)
}

#[get("osmosis/validator-commission/{address}")]
pub async fn validator_commission_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_validator_commission(&validator_address).await;

    Json(resp)
}

#[get("osmosis/validator-rewards/{address}")]
pub async fn validator_rewards_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_validator_rewards(&validator_address).await;

    Json(resp)
}

#[get("osmosis/validators-bonded")]
pub async fn validators_bonded_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_validators_bonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/validators-unbonded")]
pub async fn validators_unbonded_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_validators_unbonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/validators-unbonding")]
pub async fn validators_unbonding_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_validators_unbonding(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/validators-unspecified")]
pub async fn validators_unspecified_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_validators_unspecified(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/validators-of/{address}")]
pub async fn validators_of_delegator_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_validators_by_delegator(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair_osmosis(path: Path<(String, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (validator_address, delegator_address) = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_delegator_validator_pair_info(&delegator_address, &validator_address)
        .await;

    Json(resp)
}

// ==== SECRET ====

#[get("secret/validator/{address}")]
pub async fn validator_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_validator(&validator_address).await;

    Json(resp)
}

#[get("secret/validator-commission/{address}")]
pub async fn validator_commission_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_validator_commission(&validator_address).await;

    Json(resp)
}

#[get("secret/validator-rewards/{address}")]
pub async fn validator_rewards_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_validator_rewards(&validator_address).await;

    Json(resp)
}

#[get("secret/validators-bonded")]
pub async fn validators_bonded_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_validators_bonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/validators-unbonded")]
pub async fn validators_unbonded_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_validators_unbonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/validators-unbonding")]
pub async fn validators_unbonding_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_validators_unbonding(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/validators-unspecified")]
pub async fn validators_unspecified_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_validators_unspecified(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/validators-of/{address}")]
pub async fn validators_of_delegator_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain
        .get_validators_by_delegator(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair_secret(path: Path<(String, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (validator_address, delegator_address) = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain
        .get_delegator_validator_pair_info(&delegator_address, &validator_address)
        .await;

    Json(resp)
}

