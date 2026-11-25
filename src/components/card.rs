use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, Child, StaticClass, View, class, component, view},
};

const CARD: StaticClass = class!(
    "flex flex-col gap-5 rounded-xl border border-border bg-card py-6",
    "text-card-foreground shadow-sm",
    "has-[>[data-slot=table]:first-child]:overflow-hidden",
    "has-[>[data-slot=table]:first-child]:pt-0",
    "has-[>[data-slot=table]:last-child]:overflow-hidden",
    "has-[>[data-slot=table]:last-child]:pb-0",
);

#[component]
pub async fn card(
    cx: &Cx,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! { <div class=(class!(CARD, attrs.remove("class"))) (attrs)>(child)</div> })
}

#[component]
pub async fn card_header(
    cx: &Cx,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div
            class=(class!("flex flex-col gap-1.5 px-6", attrs.remove("class")))
            (attrs)
        >
            (child)
        </div>
    })
}

#[component]
pub async fn card_title(
    cx: &Cx,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <h3
            class=(class!("text-base leading-none font-semibold", attrs.remove("class")))
            (attrs)
        >
            (child)
        </h3>
    })
}

#[component]
pub async fn card_description(
    cx: &Cx,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <p
            class=(class!("text-sm text-muted-foreground", attrs.remove("class")))
            (attrs)
        >
            (child)
        </p>
    })
}

#[component]
pub async fn card_content(
    cx: &Cx,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! { <div class=(class!("px-6", attrs.remove("class"))) (attrs)>(child)</div> })
}

#[component]
pub async fn card_footer(
    cx: &Cx,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div
            class=(class!("flex items-center gap-2 px-6", attrs.remove("class")))
            (attrs)
        >
            (child)
        </div>
    })
}
