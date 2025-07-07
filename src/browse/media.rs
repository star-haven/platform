use crate::prelude::*;

use entity::sea_orm_active_enums::ModMediaType;
use phosphor_leptos::{Icon, IconWeight, PLUS, TRASH};
use server_fn::codec::{MultipartFormData, MultipartData};
use leptos::web_sys::{FormData, HtmlFormElement};

/// Returns the file path and URL for a given ModMedia id, assuming it is ModMediaType::Image
#[cfg(feature = "ssr")]
pub fn paths_for_image(id: Uuid) -> (std::path::PathBuf, String) {
    let filename = format!("{id}.webp");
    let mut path = crate::static_assets_dir();
    path.push("mod_media");
    let _ = std::fs::create_dir_all(&path);
    path.push(&filename);

    (path, format!("/assets/mod_media/{filename}"))
}

#[server(input = MultipartFormData)]
async fn upload_image(multipart: MultipartData) -> Result<ModMedia, ServerFnError> {
    use std::io::Cursor;
    use image::ImageReader;

    // Parse the form data
    let mut multipart = multipart.into_inner().unwrap();
    let mut image = None;
    let mut mod_id = None;
    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("image") => {
                image = Some(field.bytes().await?);
            }
            Some("mod_id") => {
                mod_id = Some(Uuid::parse_str(&field.text().await?)?);
            }
            _ => {}
        }
    }
    let Some(mod_id) = mod_id else {
        let response = expect_context::<leptos_axum::ResponseOptions>();
        response.set_status(http::status::StatusCode::BAD_REQUEST);
        return Err(ServerFnError::ServerError("Missing mod_id".to_string()));
    };
    let Some(image) = image else {
        let response = expect_context::<leptos_axum::ResponseOptions>();
        response.set_status(http::status::StatusCode::BAD_REQUEST);
        return Err(ServerFnError::ServerError("Missing image".to_string()));
    };

    super::require_session_mod_author(mod_id).await?;

    let id = Uuid::new_v4();
    let (mut path, url) = paths_for_image(id);

    // Save as WebP
    let cursor = Cursor::new(image);
    let mut image = ImageReader::new(cursor)
        .with_guessed_format()?
        .decode()?;
    if image.width() > 1920 || image.height() > 1080 {
        image = image.resize_to_fill(1920, 1080, image::imageops::FilterType::Lanczos3);
    }
    image.save(&path)?;

    // Save thumbnail
    path.set_extension("thumbnail.webp");
    image.resize_to_fill(144, 81, image::imageops::FilterType::Lanczos3).save(&path)?;

    let next_position = entity::mod_media::Entity::find()
        .filter(entity::mod_media::Column::ModId.eq(mod_id))
        .order_by_desc(entity::mod_media::Column::Position)
        .one(&db())
        .await?
        .map(|media| media.position + 1)
        .unwrap_or_default();

    use sea_orm::Set;
    let media = entity::mod_media::ActiveModel {
        id: Set(id),
        mod_id: Set(mod_id),
        media_type: Set(ModMediaType::Image),
        url: Set(url),
        position: Set(next_position),
    }.insert(&db()).await?;
    Ok(media)
}

#[server]
async fn delete_media(id: Uuid) -> Result<(), ServerFnError> {
    let Some(media) = entity::mod_media::Entity::find_by_id(id).one(&db()).await? else {
        let response = expect_context::<leptos_axum::ResponseOptions>();
        response.set_status(http::status::StatusCode::NOT_FOUND);
        return Err(ServerFnError::ServerError("Media does not exist".to_string()));
    };

    super::require_session_mod_author(media.mod_id).await?;

    if media.media_type == ModMediaType::Image {
        if let Err(error) = std::fs::remove_file(paths_for_image(id).0) {
            log::error!("error deleting media image: {error:?}")
        }
    }
    media.delete(&db()).await?;

    Ok(())
}

fn image_url_to_thumbnail_url(url: &str) -> String {
    url.replace(".webp", ".thumbnail.webp")
}

#[component]
pub fn Carousel(mod_id: Uuid, is_editing: Signal<bool>) -> impl IntoView {
    let media = Resource::new(move || (), move |_| get_mod_media(mod_id));
    let media_vec = Signal::derive(move || media.get().unwrap_or_else(|| Ok(vec![])).unwrap_or_default());
    let current_position = RwSignal::new(0);

    let delete_item = Action::new(move |id: &Uuid| {
        let id = *id;
        async move {
            let mut position_before = 0;
            for item in media_vec.get().iter() {
                if item.id == id {
                    break;
                }
                position_before = item.position;
            }

            let _ = delete_media(id).await;
            media.refetch();
            current_position.set(position_before);
        }
    });

    view! {
        <div role="group" aria-roledescription="carousel" aria-label="Gallery of screenshots">
            <ul class="w-full aspect-video bg-stone-800">
                <For
                    each=media_vec
                    key=|item| item.id
                    let(item)
                >
                    <li
                        aria-current={move || current_position.get() == item.position}
                        class="relative group"
                        class:hidden={move || current_position.get() != item.position}
                    >
                        {match item.media_type {
                            ModMediaType::Image => view! {
                                <img src=item.url class="w-full h-full object-contain" />
                            }.into_any(),
                            ModMediaType::Youtube => view! {
                                <iframe class="w-full h-full object-contain" src=format!("https://www.youtube.com/embed/{}", item.url) allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen />
                            }.into_any(),
                        }}

                        // Delete button
                        <Show when=is_editing>
                            <button
                                type="button"
                                on:click=move |_| {
                                    delete_item.dispatch(item.id);
                                }
                                title=match item.media_type {
                                    ModMediaType::Image => "Remove image",
                                    ModMediaType::Youtube => "Remove video",
                                }
                                class="opacity-0 group-hover:opacity-100 absolute top-4 right-4 p-1 bg-stone-800 bg-opacity-40 rounded flex items-center justify-center"
                            >
                                <Icon icon=TRASH weight=IconWeight::Regular size="21px" />
                            </button>
                        </Show>
                    </li>
                </For>
            </ul>
            <ul aria-label="Thumbnails" class="flex gap-2 mt-2 overflow-x-auto" class:hidden=move || media_vec.get().len() <= 1 && !is_editing.get()>
                <For
                    each=media_vec
                    key=|item| item.id
                    let(item)
                >
                    <li
                        aria-current={move || current_position.get() == item.position}
                        class="aspect-video h-20"
                    >
                        <button type="button" on:click=move |_| current_position.set(item.position)>
                            <img
                                src=match item.media_type {
                                    ModMediaType::Image => image_url_to_thumbnail_url(&item.url),
                                    ModMediaType::Youtube => format!("https://img.youtube.com/vi/{}/default.jpg", item.url),
                                }
                                class="w-full h-full object-contain"
                                fetchpriority="low"
                            />
                        </button>
                    </li>
                </For>
                <Show when=is_editing>
                    <li
                        class="aspect-video h-20 bg-stone-600"
                    >
                        <ImageUpload mod_id=mod_id resource=media current_position=current_position />
                    </li>
                </Show>
            </ul>
        </div>
    }
}

#[component]
fn ImageUpload(
    mod_id: Uuid,
    resource: Resource<Result<Vec<ModMedia>, ServerFnError>>,
    current_position: RwSignal<i32>,
) -> impl IntoView {
    let file_input = NodeRef::new();
    let upload_action = Action::new_local(move |data: &FormData| {
        let data = data.clone();
        async move {
            match upload_image(data.into()).await {
                Ok(ModMedia { position, .. }) => {
                    resource.refetch();
                    current_position.set(position);
                }
                Err(error) => log::error!("error uploading image: {error:?}"),
            }
        }
    });

    view! {
        <form class="w-full h-full" enctype="multipart/form-data" on:submit=move |ev| ev.prevent_default()>
            <input class="hidden" type="text" name="mod_id" value=mod_id.to_string() />
            <input
                class="hidden"
                type="file"
                name="image"
                accept=".avif,.bmp,.dds,.exr,.ff,.gif,.hdr,.ico,.jpeg,.jpg,.png,.pnm,.qoi,.tga,.tiff,.tif,.webp"
                node_ref=file_input
                on:change=move |_| {
                    use leptos::wasm_bindgen::JsCast;

                    let file_input = file_input.get().unwrap();
                    let form = file_input.form().unwrap().unchecked_into::<HtmlFormElement>();
                    let form_data = FormData::new_with_form(&form).unwrap();
                    upload_action.dispatch_local(form_data);
                }
            />
            <button
                type="button"
                on:click=move |ev| {
                    ev.prevent_default();
                    let file_input = file_input.get().unwrap();
                    file_input.click();
                }
                title="Add new media"
                class="w-full h-full flex items-center justify-center"
            >
                <Icon icon=PLUS weight=IconWeight::Bold size="32px" />
            </button>
        </form>
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
