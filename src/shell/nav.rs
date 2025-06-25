use crate::prelude::*;

use phosphor_leptos::{Icon, IconWeight, SIGN_IN, SIGN_OUT};

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <div class="nav-tear-shadow w-80 h-screen" />
        <nav class="sticky top-0 w-80 h-screen bg-stone-100 crumpled-paper nav-tear-mask p-4 pr-12">
            <ul class="flex flex-col gap-2 h-full">
                <li><a href="/" class="text-lg font-bold">"Star Haven"</a></li>
                <li class="mt-auto">
                    <SessionStatusBar />
                </li>
            </ul>
        </nav>
    }
}

#[server]
async fn get_session_user() -> Result<Option<User>, ServerFnError> {
    Ok(session().user().await?)
}

#[component]
fn SessionStatusBar() -> impl IntoView {
    let user = OnceResource::new_blocking(get_session_user());
    view! {
        <Suspense fallback=|| {}>
            {move || Suspend::new(async move {
                view! {
                    {move || match user.get().and_then(|r| r.ok()).flatten() {
                        Some(user) => view! {
                            <div class="border border-stone-200 rounded-full p-2 flex items-center bg-white">
                                <div class="rounded-full bg-yellow-500 w-8 h-8 mr-2" />
                                <p class="text-sm text-stone-800">{user.username}</p>
                                <button
                                    title="Log out"
                                    class="ml-auto mr-1 text-stone-500 hover:text-stone-700"
                                    on:click=move |_| {
                                        #[cfg(feature = "hydrate")]
                                        spawn_local(async move {
                                            crate::auth::logout().await.expect_throw("failed to log out");
                                            web_sys::window()
                                                .expect_throw("failed to get window")
                                                .location()
                                                .reload()
                                                .expect_throw("failed to reload page");
                                        });
                                    }
                                >
                                    <Icon icon=SIGN_OUT weight=IconWeight::Bold />
                                </button>
                            </div>
                        }.into_any(),
                        None => view! {
                            <a href="/auth" class="flex items-center gap-2 text-stone-500 hover:text-stone-700">
                                <Icon icon=SIGN_IN weight=IconWeight::Bold />
                                "sign in"
                            </a>
                        }.into_any(),
                    }}
                }
            })}
        </Suspense>
    }
}
