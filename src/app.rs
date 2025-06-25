use crate::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, Link, provide_meta_context};
use leptos_router::{
    path,
    components::{Route, Router, Routes},
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/star-haven-platform.css" />

        <Link rel="preconnect" href="https://fonts.googleapis.com" />
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="" />
        <Stylesheet href="https://fonts.googleapis.com/css2?family=Sora:wght@100..800&display=swap" />

        <Title text="Star Haven" />

        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/auth") view=crate::auth::AuthPage />
            </Routes>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <Shell>
            "Hello, world!"
        </Shell>
    }
}
