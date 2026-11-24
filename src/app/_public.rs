mod support;

use topcoat::{
    Result,
    context::Cx,
    router::{Slot, layout},
    view::{View, view},
};

#[layout]
async fn layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    Ok(view! { <div class="mx-auto max-w-2xl px-4 py-12">(slot)</div> })
}
