use actix_web::{get, web::Data, Responder};
use serde::{Deserialize, Serialize};

use crate::routes::{TNRAppError, TNRAppSuccessResponse};
use crate::state::State;

// ======== Chains Methods ========

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Chain {
    name: String,
    logo: String,
    main_denom: String,
}

#[get("chains")]
pub async fn chains(state: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chains = state
        .get_chains()
        .clone()
        .into_iter()
        .map(|(name, chain)| Chain {
            name,
            logo: chain.config.logo,
            main_denom: chain.config.main_denom,
        })
        .collect::<Vec<Chain>>();

    Ok(TNRAppSuccessResponse::new(chains))
}
