use topcoat::{
    Result,
    context::{Cx, context},
    router::{error::RouterErrorExt, href},
};

use crate::{
    domain::user::Actor,
    services::auth::{actor_for_token_hash, hash_to_hex},
};

pub async fn current_actor(cx: &Cx) -> Result<Option<Actor>> {
    let Some(hash) = topcoat::session::token_hash(cx).await? else {
        return Ok(None);
    };

    let mut db = context::<toasty::Db>(cx).clone();
    let token_hash = hash_to_hex(&hash);
    Ok(actor_for_token_hash(&mut db, &token_hash).await?)
}

pub async fn require_actor(cx: &Cx) -> Result<Actor> {
    let actor = current_actor(cx)
        .await?
        .ok_or_redirect(href!(crate::app::login::page).resolve(cx))?;
    Ok(actor)
}
