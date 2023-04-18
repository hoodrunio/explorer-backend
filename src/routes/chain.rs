use actix_web::{
    get,
    web::{Data, Path},
    Responder,
};
use serde::{Deserialize, Serialize};

use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};
use crate::state::State;

// ======== Chains Methods ========

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChainResponseItem {
    name: String,
    logo: String,
    main: bool,
    main_denom: String,
    main_symbol: String,
    acc_address_prefix: String,
}

#[get("chains")]
pub async fn chains(state: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chains = state
        .get_chains()
        .clone()
        .into_iter()
        .map(|(name, chain)| ChainResponseItem {
            name,
            logo: chain.config.logo,
            main: chain.config.main,
            main_denom: chain.config.main_denom,
            main_symbol: chain.config.main_symbol,
            acc_address_prefix: chain.config.base_prefix,
        })
        .collect::<Vec<ChainResponseItem>>();

    Ok(TNRAppSuccessResponse::new(chains, None))
}

#[get("{chain}/dashboard")]
pub async fn dashboard(path: Path<String>, chains_data: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();
    let chain = extract_chain(&chain, chains_data)?;

    let data = chain.get_dashboard_info().await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/stats")]
pub async fn stats(path: Path<String>, chains_data: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();
    let chain = extract_chain(&chain, chains_data)?;

    let data = chain.get_stats().await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}
