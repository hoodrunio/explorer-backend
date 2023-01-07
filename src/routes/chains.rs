use crate::routes::OutRestResponse;
use crate::{fetch::others::Response, state::State};
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};
use serde_json::json;
use serde::{Deserialize, Serialize};

// ======== Chains Methods ========

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Chain{
    name: String,
    logo: String,
    main_denom: String,
}

#[get("chains")]
pub async fn chains(state: Data<State>) -> impl Responder {
    let mut chains  = state.get_chains().clone().into_iter().map(|(name, chain)|
        Chain{
            name: name,
            logo: chain.config.logo,
            main_denom: chain.config.main_denom,
        }
    ).collect::<Vec<Chain>>();
    Json(Response::Success(chains))
}
