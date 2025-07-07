use crate::prelude::*;

use leptos::Params;
use leptos_router::{hooks::{query_signal, use_params}, params::Params};

mod media;

#[derive(Params, PartialEq)]
struct ModPageParams {
    slug: Option<String>,
}

#[component]
pub fn ModPage() -> impl IntoView {
    let params = use_params::<ModPageParams>();
    let slug = Signal::derive(move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.slug.clone())
            .unwrap_or_default()
    });

    let (is_editing, _) = query_signal::<String>("edit");
    let is_editing = Signal::derive(move || is_editing.get().is_some());

    let mod_data = Resource::new_blocking(
        move || slug.get(),
        get_mod_by_slug 
    );

    view! {
        <Shell>
            <Suspense fallback=|| {}>
                {move || match mod_data.get() {
                    Some(Ok((mod_data, is_author))) => {
                        let initial_data = mod_data.clone();
                        view! {
                            <Show when=move || is_author>
                                <AuthorToolbar
                                    slug=mod_data.slug.clone()
                                    is_published=Signal::derive(move || mod_data.published_at.is_some())
                                />
                            </Show>
                            <div class="w-full max-w-screen-md mx-auto my-16">
                                <ModForm initial_data=initial_data is_editing=is_editing />
                            </div>
                        }.into_any()
                    }
                    Some(Err(ServerFnError::ServerError(s))) if s == "Mod not found" => view! {
                        <div class="w-full max-w-screen-md mx-auto my-16 text-center">
                            <h1 class="text-xl font-semibold mb-4">"We couldn't find this mod"</h1>
                            <a href="/browse" class="text-yellow-400 underline">Go back to browsing</a>
                        </div>
                    }.into_any(),
                    _ => ().into_any(),
                }}
            </Suspense>
        </Shell>
    }
}

#[server]
async fn get_mod_by_slug(slug: String) -> Result<(Mod, bool), ServerFnError> {
    let mut condition = sea_orm::Condition::any()
        .add(entity::mods::Column::PublishedAt.is_not_null());

    // Users can view unpublished mods if they are authors of them
    if let Some(user) = session().user().await? {
        condition = condition.add(entity::mod_authors::Column::UserId.eq(user.id));
    }

    if session().has_scope(crate::auth::Scope::AdminAuthorAllMods) {
        condition = condition.add(entity::mod_authors::Column::UserId.is_not_null()); // Always true
    }

    let Some(ret) = Mods::find()
        .filter(entity::mods::Column::Slug.eq(slug))
        .join(JoinType::InnerJoin, entity::mods::Relation::ModAuthors.def())
        .filter(condition)
        .one(&db())
        .await?
    else {
        let response = expect_context::<leptos_axum::ResponseOptions>();
        response.set_status(http::status::StatusCode::NOT_FOUND);
        return Err(ServerFnError::ServerError("Mod not found".to_string()));
    };

    let is_author = is_session_mod_author(ret.id).await?;
    Ok((ret, is_author))
}

#[cfg(feature = "ssr")]
async fn is_session_mod_author(mod_id: Uuid) -> Result<bool, ServerFnError> {
    let Some(user) = session().user().await? else { return Ok(false); };

    if session().has_scope(crate::auth::Scope::AdminAuthorAllMods) {
        return Ok(true);
    }

    let author = ModAuthors::find()
        .filter(entity::mod_authors::Column::ModId.eq(mod_id))
        .filter(entity::mod_authors::Column::UserId.eq(user.id))
        .one(&db())
        .await?;
    Ok(author.is_some())
}

#[cfg(feature = "ssr")]
async fn require_session_mod_author(mod_id: Uuid) -> Result<(), ServerFnError> {
    if is_session_mod_author(mod_id).await? {
        Ok(())
    } else {
        let response = expect_context::<leptos_axum::ResponseOptions>();
        response.set_status(http::status::StatusCode::UNAUTHORIZED);
        Err(ServerFnError::ServerError("No permission".to_string()))
    }
}

#[component]
pub fn AuthorToolbar(
    slug: String,
    is_published: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="w-full p-4 pl-12 bg-stone-800 shadow-lg flex items-center">
            <a href=format!("/mod/{}?edit", slug) class="font-semibold text-lg">
                Edit mod
            </a>
            <button class="ml-auto bg-green-600 text-white font-semibold select-none shadow-sm py-2 px-3 rounded inline-flex items-center">
                {move || if is_published.get() { "Published" } else { "Publish" }}
            </button>
        </div>
    }
}

#[component]
pub fn ModForm(initial_data: Mod, is_editing: Signal<bool>) -> impl IntoView {
    let edit_mod = ServerAction::<EditMod>::new();

    let name = RwSignal::new(initial_data.name);
    let description = RwSignal::new(initial_data.description);

    let game = OnceResource::new_blocking(get_game(initial_data.game_id));

    view! {
        <ActionForm action=edit_mod>
            <input type="text" name="id" value=initial_data.id.to_string() class="hidden" />

            <Transition fallback=move || ()>
                <media::Carousel mod_id=initial_data.id is_editing=is_editing />
            </Transition>

            <div class="bg-stone-800 p-4 mt-4">
                <Show when=is_editing fallback=move || view! {
                    <h1 class="text-3xl text-white font-semibold mb-4">
                        {name}
                    </h1>
                }>
                    <input type="text" name="name" bind:value=name placeholder="Title" required maxlength=30 class="text-3xl text-white font-semibold mb-4 w-full bg-transparent" />
                </Show>

                <div class="text-stone-500 flex flex-row gap-8"> 
                    <Suspense fallback=move || ()>
                        <span>
                            "System: "
                            {move || match game.get() {
                                Some(Ok(game)) => game.console_name,
                                _ => "".to_string(),
                            }}
                        </span>
                    </Suspense>
                    <span>
                        "Release date: "
                        {match initial_data.published_at {
                            Some(date) => view! { <LocaleDate date=Signal::derive(move || date) /> }.into_any(),
                            None => view! { "not published" }.into_any(),
                        }}
                    </span>
                </div>
            </div>

            // TODO: markdown
            <Show when=is_editing fallback=move || view! {
                <p class="whitespace-pre-wrap my-4 text-stone-200 text-md">{description}</p>
            }>
                <textarea
                    name="description"
                    prop:value=move || description.get()
                    on:input:target=move |ev| description.set(ev.target().value())
                    class="block p-2 my-4 border-2 border-stone-500 text-stone-200 bg-stone-700 text-md w-full rounded-sm"
                >
                    {description}
                </textarea>
            </Show>

            <Show when=is_editing>
                <super::create::ActionFormSubmitButton
                    pending=edit_mod.pending()
                    error=Signal::derive(move || edit_mod.value().get().and_then(Result::err))
                >
                    "Save"
                </super::create::ActionFormSubmitButton>
            </Show>
        </ActionForm>
    }
}

#[server]
async fn get_game(id: Uuid) -> Result<Game, ServerFnError> {
    let game = Games::find_by_id(id).one(&db()).await?;
    if let Some(game) = game {
        Ok(game)
    } else {
        let response = expect_context::<leptos_axum::ResponseOptions>();
        response.set_status(http::status::StatusCode::NOT_FOUND);
        Err(ServerFnError::ServerError("Game not found".to_string()))
    }
}

#[server]
pub async fn edit_mod(id: Uuid, name: String, description: String) -> Result<(), ServerFnError> {
    use sea_orm::Set;

    require_session_mod_author(id).await?;

    entity::mods::ActiveModel {
        id: Set(id),
        name: Set(name),
        description: Set(description),
        ..Default::default()
    }.save(&db()).await?;

    Ok(())
}

use time::{format_description::{FormatItem, well_known::Rfc3339}, OffsetDateTime};

/// Date format used by SSR where the locale is not known
static DATE_FORMAT: &[FormatItem<'static>] = time::macros::format_description!(version = 2, "[year]-[month]-[day]");

/// Renders a date, without its time, using the browser locale
#[component]
pub fn LocaleDate(date: Signal<OffsetDateTime>) -> impl IntoView {
    let formatted = Signal::derive(move || {
        date.get().format(DATE_FORMAT).unwrap_or_else(|_| "Invalid date".to_string())
    });
    let rfc3339 = Signal::derive(move || date.get().format(&Rfc3339).unwrap_or_default());

    let node_ref: NodeRef<leptos::html::Time> = NodeRef::new();

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        if let Some(el) = node_ref.get() {
            let js_date = js_sys::Date::new(&js_sys::JsString::from(rfc3339.get()));
            let locale_str = js_date.to_locale_date_string("", &wasm_bindgen::JsValue::undefined());
            let locale_str = JsValue::from(locale_str).as_string();
            el.set_text_content(locale_str.as_deref());
        }
    });

    view! {
        <time
            node_ref=node_ref
            datetime=rfc3339
        >
            {formatted}
        </time>
    }
}
