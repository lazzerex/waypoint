use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, context},
    router::{
        content::Form,
        error::{SeeOther, see_other},
        href,
        page,
        request,
    },
    session,
    view::{View, view},
};

use crate::{
    app::{_staff::tickets, auth},
    services::auth::{self as auth_service, hash_to_hex, session_expiry},
};

#[derive(Deserialize)]
pub struct LoginInput {
    email: String,
    password: String,
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    if auth::current_actor(cx).await?.is_some() {
        return Err(see_other(href!(tickets::page).resolve(cx)).into());
    }

    Ok(login_form(cx, "", None))
}

#[page(POST)]
pub async fn login(cx: &Cx, Form(input): Form<LoginInput>) -> Result<impl View> {
    let mut db = context::<toasty::Db>(cx).clone();

    let actor = match auth_service::authenticate(&mut db, &input.email, &input.password).await {
        Ok(Some(actor)) => actor,
        Ok(None) => return Ok(login_form(cx, &input.email, Some("Email or password is incorrect."))),
        Err(error) => {
            eprintln!("login failed: {error}");
            return Ok(login_form(
                cx,
                &input.email,
                Some("Sign in is temporarily unavailable."),
            ));
        }
    };

    let session = session::start(cx).await?;
    let Some(expires_at) = session_expiry(session.expires_at) else {
        return Err(topcoat::Error::msg("session expiry is invalid"));
    };
    let token_hash = hash_to_hex(&session.token_hash);
    let user_agent = request::headers(cx)
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let ip = request::client_ip(cx).map(|ip| ip.to_string());

    if let Err(error) = auth_service::create_session(
        &mut db,
        actor.id,
        &token_hash,
        expires_at,
        user_agent,
        ip,
    )
    .await
    {
        eprintln!("session creation failed: {error}");
        return Ok(login_form(
            cx,
            &input.email,
            Some("Sign in is temporarily unavailable."),
        ));
    }

    Err(see_other(href!(tickets::page).resolve(cx)).into())
}

fn login_form(cx: &Cx, email: &str, error: Option<&str>) -> impl View {
    view! {
        <main class="mx-auto max-w-sm px-4 py-16">
            <h1 class="text-2xl font-semibold tracking-tight">"Staff sign in"</h1>
            if let Some(error) = error {
                <p class="mt-4 rounded-md border border-border bg-muted px-4 py-3 text-sm">
                    (error)
                </p>
            }
            <form method="post" action=(href!(login)) class="mt-6 flex flex-col gap-4">
                <label class="flex flex-col gap-2 text-sm">
                    "Email"
                    <input
                        class="rounded-md border border-border bg-background px-3 py-2"
                        type="email"
                        name="email"
                        value=(email)
                        autocomplete="username"
                        required
                    >
                </label>
                <label class="flex flex-col gap-2 text-sm">
                    "Password"
                    <input
                        class="rounded-md border border-border bg-background px-3 py-2"
                        type="password"
                        name="password"
                        autocomplete="current-password"
                        required
                    >
                </label>
                <button
                    class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground"
                    type="submit"
                >
                    "Sign in"
                </button>
            </form>
        </main>
    }
}
