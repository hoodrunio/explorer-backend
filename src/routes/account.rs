use crate::state::State;
use actix_web::{
    get,
    web::{Data, Path},
    Responder,
};

use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};

// ======== Account Methods ========

//account address => evmos198zkgedxs9f77ru80zd3g693dhpv6n5wej6d7p
#[get("{chain}/account/{account_address}")]
pub async fn account(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, account_address) = path.into_inner();
    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_account_info(&account_address).await?;
    Ok(TNRAppSuccessResponse::new(data))
}
