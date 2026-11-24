use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <h1 class="text-2xl font-semibold tracking-tight">"Contact support"</h1>
        <p class="mt-2 text-muted-foreground">
            "The public ticket form arrives in Phase 1."
        </p>
    })
}
