use super::QueryParams;
use crate::routes::OutRestResponse;
use crate::{
    fetch::others::{PaginationConfig, Response},
    state::State,
};
use actix_web::{
    get,
    web::{Data, Json, Path, Query},
    Responder,
};

// ======== Tokenomic Methods ========

#[get("{chain}/supply/{denom}")]
pub async fn supply(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, denom) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_supply_by_denom(&denom).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/supplies")]
pub async fn supplies(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(60).page(query.page.unwrap_or(1));

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_supply_of_all_tokens(config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/inflation")]
pub async fn inflation(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => match chain.inner.data.inflation.lock() {
            Ok(inflation_rate) => Response::Success(OutRestResponse {
                pages: 0,
                value: *inflation_rate,
            }),
            Err(_) => Response::Error("Cannot return inflation rate.".to_string()),
        },
        Err(err) => Response::Error(err),
    })
}
