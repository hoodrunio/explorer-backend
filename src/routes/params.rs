use actix_web::{
    get,
    Responder,
    web::{Data, Path},
};

use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};
use crate::state::State;

// ======== Parameter Methods ========

#[get("{chain}/params")]
pub async fn params(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    // Database can be used.
    let data = chain.get_params_all().await?;
    Ok(TNRAppSuccessResponse::new(data))
}
