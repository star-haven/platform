use std::thread::current;

use crate::prelude::*;

use entity::sea_orm_active_enums::ModMediaType;
use leptos::Params;
use leptos_router::{hooks::use_params, params::Params};

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
    let mod_data = Resource::new_blocking(
        move || slug.get(),
        |slug| get_mod_by_slug(slug) 
    );
    view! {
        <Shell>
            <Suspense fallback=|| {}>
                {move || match mod_data.get() {
                    Some(Ok(mod_data)) => view! {
                        <div class="w-full max-w-screen-md mx-auto my-16">
                            <MediaCarousel mod_id={mod_data.id} />
                            <h1 class="text-xl font-semibold my-4">{mod_data.name}</h1>
                            <p>{mod_data.description}</p>
                        </div>
                    }.into_any(),
                    Some(Err(ServerFnError::ServerError(s))) if s == "Mod not found" => view! {
                        <div class="w-full max-w-screen-md mx-auto my-16 text-center">
                            <h1 class="text-xl font-semibold mb-4">"We couldn't find this mod"</h1>
                            <a href="/browse" class="text-yellow-400 underline">Go back to browsing</a>
                        </div>
                    }.into_any(),
                    _ => view! {}.into_any(),
                }}
            </Suspense>
        </Shell>
    }
}

#[server]
async fn get_mod_by_slug(slug: String) -> Result<Mod, ServerFnError> {
    let mut condition = sea_orm::Condition::any()
        .add(entity::mods::Column::PublishedAt.is_not_null());

    // Users can view unpublished mods if they are authors of them
    if let Some(user) = session().user().await? {
        condition = condition.add(entity::mod_authors::Column::UserId.eq(user.id));
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
    Ok(ret)
}

#[component]
pub fn MediaCarousel(mod_id: Uuid) -> impl IntoView {
    let media = OnceResource::new(get_mod_media(mod_id));
    let current_position = RwSignal::new(0);
    view! {
        <div role="group" aria-roledescription="carousel" aria-label="Gallery of screenshots">
            {move || Suspend::new(async move {
                match media.await {
                    Ok(media) => view! {
                        <ul>
                            {media.clone().into_iter().map(move |media| {
                                view! {
                                    <li
                                        aria-current={move || current_position.get() == media.position}
                                        class="aspect-video"
                                        class:hidden={move || current_position.get() != media.position}
                                    >
                                        {match media.media_type {
                                            ModMediaType::Image => view! {
                                                <img src=media.url class="w-full h-full object-contain" />
                                            }.into_any(),
                                            ModMediaType::Youtube => view! {
                                                <iframe class="w-full h-full object-contain" src=format!("https://www.youtube.com/embed/{}", media.url) allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen />
                                            }.into_any(),
                                        }}
                                    </li>
                                }.into_any()
                            }).collect::<Vec<_>>()}
                        </ul>
                        <ul aria-label="Thumbnails" class="flex">
                            {media.clone().into_iter().map(move |media| {
                                let src = match media.media_type {
                                    ModMediaType::Image => media.url,
                                    ModMediaType::Youtube => format!("https://img.youtube.com/vi/{}/default.jpg", media.url),
                                };
                                view! {
                                    <li
                                        aria-current={move || current_position.get() == media.position}
                                        class="aspect-video"
                                    >
                                        <button on:click=move |_| current_position.set(media.position)>
                                            <img src=src class="w-full h-full object-contain" fetchpriority="low" />
                                        </button>
                                    </li>
                                }.into_any()
                            }).collect::<Vec<_>>()}
                        </ul>
                    }.into_any(),
                    Err(error) => {
                        log::error!("failed to load mod media for carousel: {:?}", error);
                        view! {}.into_any()
                    }
                }
            })}
        </div>
    }
}

#[server]
async fn get_mod_media(mod_id: Uuid) -> Result<Vec<ModMedia>, ServerFnError> {
    let media = entity::mod_media::Entity::find()
        .filter(entity::mod_media::Column::ModId.eq(mod_id))
        .order_by_asc(entity::mod_media::Column::Position)
        .all(&db())
        .await?;
    Ok(media)
}
