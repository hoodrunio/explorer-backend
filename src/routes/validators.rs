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

// ======== Validator Methods ========

#[get("{chain}/validator/{address}")]
pub async fn validator(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, validator_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validator_info(&validator_addr).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validator-delegations/{address}")]
pub async fn validator_delegations(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let (chain, validator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1));

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validator_delegations(&validator_addr, config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validator-unbondings/{address}")]
pub async fn validator_unbondings(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let (chain, validator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1));

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validator_unbondings(&validator_addr, config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validator-redelegations/{address}")]
pub async fn validator_redelegations(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let (chain, validator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(6).page(query.page.unwrap_or(1));

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validator_redelegations(&validator_addr, config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validator-commission/{address}")]
pub async fn validator_commission(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, validator_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validator_commission(&validator_addr).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validator-rewards/{address}")]
pub async fn validator_rewards(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, validator_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validator_rewards(&validator_addr).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validators-bonded")]
pub async fn validators_bonded(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validators_bonded(PaginationConfig::new()).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validators-unbonded")]
pub async fn validators_unbonded(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validators_unbonded(PaginationConfig::new()).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validators-unbonding")]
pub async fn validators_unbonding(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validators_unbonding(PaginationConfig::new()).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validators-unspecified")]
pub async fn validators_unspecified(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validators_unspecified(PaginationConfig::new()).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validators-of/{address}")]
pub async fn validators_of_delegator(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, delegator_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validators_by_delegator(&delegator_addr, PaginationConfig::new()).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair(path: Path<(String, String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, validator_addr, delegator_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_delegator_validator_pair_info(&delegator_addr, &validator_addr).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validator-set/{height}")]
pub async fn validator_set_by_height(path: Path<(String, u64)>, chains: Data<State>) -> impl Responder {
    let (chain, height) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validator_set_by_height(height).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/validator-set")]
pub async fn validator_set(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validator_set().await.into(),
        Err(err) => Response::Error(err),
    })
}
