use topcoat::{
    Result,
    context::{Cx, context},
    router::{
        error::{SeeOther, see_other},
        href, route,
    },
    session,
};

use crate::services::auth::{delete_session, hash_to_hex};

#[route(POST)]
pub async fn logout(cx: &Cx) -> Result<SeeOther> {
    if let Some(hash) = session::stop(cx).await? {
        let mut db = context::<toasty::Db>(cx).clone();
        let token_hash = hash_to_hex(&hash);
        delete_session(&mut db, &token_hash).await?;
    }

    Ok(see_other(href!(crate::app::home).resolve(cx)))
}
