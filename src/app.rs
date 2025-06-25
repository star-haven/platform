use crate::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
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
        <PageShell>
            "Hello, world!"
        </PageShell>
    }
}

#[server]
async fn get_username() -> Result<Option<String>, ServerFnError> {
    Ok(session().user().await?.map(|user| user.username))
}

#[component]
pub fn PageShell(children: Children) -> impl IntoView {
    view! {
        <div>
            <SiteNav />
            <main>
                {children()}
            </main>
            <footer>
                <p>
                    "I am the footer"
                </p>
            </footer>
        </div>
    }
}

#[server]
async fn get_session_user() -> Result<Option<User>, ServerFnError> {
    Ok(session().user().await?)
}

#[component]
fn SiteNav() -> impl IntoView {
    let user = OnceResource::new_blocking(get_session_user());
    let user = move || user.get().expect("user").ok().flatten();

    view! {
        <nav>
            "Star Haven"
        
            <Suspense fallback=|| {}>
                {move || Suspend::new(async move {
                    view! {
                        <Show when=move || user().is_some()>
                            {move || user().unwrap().username}
                        </Show>
                        <Show when=move || user().is_none()>
                            <a href="/auth">Log in</a>
                        </Show>
                    }
                })}
            </Suspense>
        </nav>
    }
}
