use actix_web::{
    get,
    Responder,
    web::{Data, Path},
};

use crate::state::State;
use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};

// ======== Signing Information Methods ========

#[get("{chain}/signing/{cons_address}")]
pub async fn signing(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, cons_addr) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_validator_signing_info(&cons_addr).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}
