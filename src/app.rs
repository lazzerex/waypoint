mod _public;
mod _staff;

use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    font::{Font, fontsource::fontsource_font},
    router::{RouterBuilder, RouterBuilderDiscoverExt, Slot, href, layout, module_router, page},
    runtime::RuntimeLayer,
    tailwind,
    view::{View, view},
};

use crate::components::card::{card, card_content, card_description, card_header, card_title};

/// The Geist font family, loaded from Fontsource.
const GEIST: Font = fontsource_font!(GEIST, weight: [400, 700], style: Normal);

/// Builds the application's router.
pub fn router() -> RouterBuilder {
    module_router!()
        .discover()
        .assets(AssetBundle::load().unwrap())
        .layer(RuntimeLayer::new())
}

#[layout]
async fn root(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"Waypoint"</title>
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                topcoat::font::link(font: GEIST)
                topcoat::runtime::script()
                topcoat::dev::script()
            </head>
            <body class="min-h-screen bg-background text-foreground">
                <header class="border-b border-border">
                    <nav class="mx-auto flex max-w-5xl items-center gap-6 px-4 py-3">
                        <a class="font-semibold" href=(href!(home))>"Waypoint"</a>
                        <a
                            class="text-sm text-muted-foreground"
                            href=(href!(crate::app::_staff::tickets::page))
                        >
                            "Tickets"
                        </a>
                    </nav>
                </header>
                (slot)
            </body>
        </html>
    })
}

#[page]
async fn home(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-2xl px-4 py-16">
            card(
                card_header(
                    card_title("Waypoint")
                    card_description("A shared inbox and helpdesk for small teams.")
                )
                card_content(
                    <p class="text-sm text-muted-foreground">
                        "Phase 0 foundation is running."
                    </p>
                )
            )
        </main>
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use topcoat::router::{Body, ModulePage};
    use topcoat::view::ViewExt;

    #[tokio::test]
    async fn home_page_renders() {
        let cx = Cx::new();
        let view = home.render(&cx, Body::empty());
        let html = view.single().await.unwrap().render(&cx);

        assert!(html.contains("Waypoint"));
        assert!(html.contains("Phase 0 foundation is running."));
    }
}
