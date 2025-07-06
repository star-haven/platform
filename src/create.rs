use crate::prelude::*;

use phosphor_leptos::{Icon, IconWeight, SPINNER_GAP, WARNING};

#[component]
pub fn DashboardPage() -> impl IntoView {
    let mods = OnceResource::new_blocking(session_mods());
    view! {
        <Shell>
            <div class="w-full max-w-screen-lg mx-auto my-8">
                <a href="https://docs.starhaven.dev/">Learn to create mods</a>

                <Suspense fallback=|| {}>
                    {move || match mods.get() {
                        Some(Ok(Some(mods))) => view! {
                            <h3>Your mods</h3>
                            <ul class="grid grid-cols-4 gap-4 my-4">
                                <For
                                    each=move || mods.clone()
                                    key=|el| el.id
                                    children=move |el| {
                                        view! {
                                            <li class="flex aspect-video bg-stone-800">
                                                <a href=format!("/mod/{}", el.slug) class="w-full h-full">
                                                    <span class="sr-only">{el.name}</span>
                                                </a>
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                            <LinkButton href="/create/new">Create a new mod</LinkButton>
                        }.into_any(),
                        Some(Err(error)) => view! {
                            <h3>Your mods</h3>
                            <p>Error loading mods: {error.to_string()}</p>
                        }.into_any(),
                        None | Some(Ok(None)) => view! {}.into_any(),
                    }}
                </Suspense>
            </div>
        </Shell>
    }
}

#[server]
async fn session_mods() -> Result<Option<Vec<Mod>>, ServerFnError> {
    let Some(user) = session().user().await? else { return Ok(None); };
    let mods = Mods::find()
        .join(JoinType::InnerJoin, entity::mods::Relation::ModAuthors.def())
        .filter(entity::mod_authors::Column::UserId.eq(user.id))
        .order_by_with_nulls(entity::mods::Column::PublishedAt, Order::Desc, NullOrdering::First)
        .all(&db())
        .await?;
    Ok(Some(mods))
}

#[component]
pub fn LinkButton(href: &'static str, children: Children) -> impl IntoView {
    view! {
        <a href={href} class="bg-yellow-600 text-white font-semibold select-none shadow-sm py-2 px-3 rounded inline-flex items-center justify-center gap-2">
            {children()}
        </a>
    }
}

/// A card that prompts the user to sign in. If they are already signed in, nothing is rendered.
#[component]
fn SessionRequiredBanner() -> impl IntoView {
    let is_logged_in = OnceResource::new_blocking(crate::auth::is_logged_in());
    view! {
        <Suspense fallback=|| {}>
            <Show when={move || matches!(is_logged_in.get(), Some(Ok(false)))}>
                <section class="bg-stone-600 border border-stone-500 text-stone-200 p-4 rounded-md mb-4" role="alert">
                    <h4 class="font-bold">"Sign in required"</h4>
                    <p class="my-2">"You must be signed in to perform this action."</p>
                    // TODO: ?return={this page}
                    <LinkButton href="/auth">Sign in</LinkButton>
                </section>
            </Show>
        </Suspense>
    }
}

/// Convert a string into a slug version of it, where alphanumerics and hyphens only are allowed
fn to_slug(name: &str) -> String {
    let slug = name
        .to_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != ' ', "")
        .replace(' ', "-");

    // Collapse multiple hyphens
    let mut collapsed = String::with_capacity(slug.len());
    let mut last_was_hyphen = false;
    for c in slug.chars() {
        if c == '-' {
            if !last_was_hyphen {
                collapsed.push('-');
                last_was_hyphen = true;
            }
        } else {
            collapsed.push(c);
            last_was_hyphen = false;
        }
    }

    collapsed.trim_matches('-').to_string()
}

#[component]
pub fn NewModPage() -> impl IntoView {
    let new_mod = ServerAction::<NewMod>::new();

    let name = RwSignal::new("".to_string());
    let default_slug = move || to_slug(&name.get());

    let games = OnceResource::new_blocking(get_all_games());

    view! {
        <Shell>
            <div class="w-full max-w-screen-md mx-auto my-6">
                <SessionRequiredBanner/>
                <h1 class="text-2xl font-bold mb-8">"Create a new mod"</h1>
                <ActionForm action=new_mod>
                    <label class="block mb-8">
                        <span class="font-semibold">"Title"</span>
                        <input type="text" name="name" bind:value=name required maxlength=30 class="block p-2 my-2 border-2 border-stone-500 text-stone-200 bg-stone-700 text-xl w-full rounded-sm" />
                    </label>
                    <label class="block mb-8">
                        <span class="font-semibold">"URL"</span>
                        <div class="flex items-stretch my-2 border-2 border-stone-500 bg-stone-700 text-base w-full rounded-sm">
                            <span class="text-stone-400 select-none py-2 pl-2" aria-hidden="true">"https://starhaven.dev/mod/"</span>
                            <input
                                type="text" name="slug" placeholder=default_slug
                                autocomplete="off" pattern="^[a-z0-9]+(?:-[a-z0-9]+)*$" minlength=3 maxlength=30 title="Only lowercase letters, numbers, and hyphens"
                                class="text-stone-200 placeholder-stone-300 bg-transparent grow py-2 pr-2"
                            />
                        </div>
                    </label>
                    <div class="block mb-8">
                        <span class="font-semibold">"Base game"</span>
                        <Suspense fallback={move || view! { "Loading games..." }}>
                            <ol>
                                <For
                                    each=move || games.get().unwrap_or(Err(ServerFnError::ServerError(String::new()))).unwrap_or_default()
                                    key=|game| game.id
                                    children=move |game| {
                                        view! {
                                            <li class="flex items-center">
                                                <input type="radio" name="game" id={game.id.to_string()} value={game.id.to_string()} required />
                                                <label for={game.id.to_string()} class="py-0.5 px-2 grow">
                                                    {game.name}
                                                    " "
                                                    <span class="text-stone-400">"(" {game.console_name} ")"</span>
                                                </label>
                                            </li>
                                        }
                                    }
                                />
                            </ol>
                        </Suspense>
                    </div>
                    <label class="block mb-8">
                        <span class="font-semibold">"Description"</span>
                        // TODO: markdown editor
                        <textarea name="description" class="block p-2 my-2 border-2 border-stone-500 text-stone-200 bg-stone-700 text-sm w-full rounded-sm" />
                    </label>
                    <ActionFormSubmitButton pending=new_mod.pending() error=Signal::derive(move || new_mod.value().get().map(Result::err).flatten())>"Save & view page"</ActionFormSubmitButton>
                </ActionForm>
            </div>
        </Shell>
    }
}

/// A submit button for actions that takes on pending and error states based on the action.
#[component]
pub fn ActionFormSubmitButton(
    #[prop(into)]
    pending: Signal<bool>,
    #[prop(into)]
    error: Signal<Option<ServerFnError>>,
    children: Children
) -> impl IntoView {
    view! {
        <div class="flex items-center">
            <button type="submit" class="bg-yellow-600 text-white font-semibold select-none shadow-sm py-2 px-3 rounded inline-flex items-center justify-center gap-2">
                {children()}
            </button>
            <Show when={move || pending.get()}>
                <span class="animate-spin inline-flex items-center justify-center ml-4"><Icon icon=SPINNER_GAP weight=IconWeight::Regular size="32px" /></span>
            </Show>
        </div>
        <Show when={move || error.get().is_some() && !pending.get()}>
            <p class="text-red-300 mt-4 flex items-center gap-2">
                <Icon icon=WARNING weight=IconWeight::Fill size="21px" />
                {move || match error.get() {
                    Some(ServerFnError::ServerError(message)) => view! { {message} }.into_any(),
                    error => {
                        log::error!("{:?}", error);
                        view! { "Something went wrong, please try again" }.into_any()
                    }
                }}
            </p>
        </Show>
    }
}

#[server]
async fn new_mod(
    slug: Option<String>,
    name: String,
    description: String,
    game: Uuid,
) -> Result<Mod, ServerFnError> {
    let Some(user) = session().user().await? else {
        return Err(ServerFnError::ServerError("Must be signed in".to_string()))
    };
    if !session().has_scope(crate::auth::Scope::CreateMod) {
        return Err(ServerFnError::ServerError("You do not have permission to create mods".to_string()));
    }

    let slug = slug.unwrap_or_else(|| to_slug(&name));

    let new_mod = db().transaction::<_, Mod, anyhow::Error>(|txn| {
        Box::pin(async move {
            use sea_orm::Set;

            let new_mod = entity::mods::ActiveModel {
                id: Set(Uuid::new_v4()),
                slug: Set(slug),
                name: Set(name),
                description: Set(description),
                game_id: Set(game),
                ..Default::default()
            }.insert(txn).await?;
            entity::mod_authors::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user.id),
                mod_id: Set(new_mod.id),
            }.insert(txn).await?;

            Ok(new_mod)
        })
    }).await?;
    leptos_axum::redirect(&format!("/mod/{}", new_mod.slug));
    Ok(new_mod)
}

#[server]
async fn get_all_games() -> Result<Vec<Game>, ServerFnError> {
    Games::find().all(&db()).await.map_err(Into::into)
}
