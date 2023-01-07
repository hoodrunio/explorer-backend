use crate::routes::{extract_chain, OutRestResponse, TNRAppError, TNRAppSuccessResponse};
use crate::{fetch::others::Response, state::State};

use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ======== Community Pool Methods ========

#[get("{chain}/community-pool")]
pub async fn community_pool(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_community_pool().await?;
    Ok(TNRAppSuccessResponse::new(data))
}
