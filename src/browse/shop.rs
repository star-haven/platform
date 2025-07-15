use crate::prelude::*;

#[component]
pub fn ShopPage() -> impl IntoView {
    let mods = OnceResource::new_blocking(published_mods_by_recency());
    view! {
        <Shell>
            <div class="w-full max-w-screen-lg mx-auto my-8">
                <Suspense fallback=|| {}>
                    <ul class="grid grid-cols-4 gap-4 my-4">
                    <For
                        each=move || mods.get().and_then(|result| result.ok()).unwrap_or_default()
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
                </Suspense>
            </div>
        </Shell>
    }
}

// TODO: pagination
#[server]
async fn published_mods_by_recency() -> Result<Vec<Mod>, ServerFnError> {
    let mods = Mods::find()
        .filter(entity::mods::Column::PublishedAt.is_not_null())
        .order_by_desc(entity::mods::Column::PublishedAt)
        .all(&db())
        .await?;
    Ok(mods)
}
