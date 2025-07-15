use sea_orm::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::Deserialize;
use time::{OffsetDateTime, UtcOffset};

#[allow(non_snake_case, unused)]
#[derive(Deserialize, Debug)]
struct ModData {
    displayName: String,
    internalName: String,
    tagline: String,
    description: String,
    creators: Vec<String>,
    releaseDate: String,
    lastUpdated: String,
    version: String,
    pageUrl: Option<String>,
    sourceUrl: Option<String>,
    iconUrl: Option<String>,
    downloadUrl: String,
    game: String,
    console: String,
    consoleCompatible: bool,
    recommendedEmulator: String,
    modGroup: Option<String>,
    color: Option<String>,
}

fn parse_date(s: &str) -> OffsetDateTime {
    let format = time::macros::format_description!("[day]/[month]/[year]");
    let date = time::Date::parse(s, &format).expect("bad date");
    date.with_hms(0, 0, 0)
        .expect("valid time")
        .assume_offset(UtcOffset::UTC)
}

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL to be set");
    let db = sea_orm::Database::connect(db_url).await.expect("to be able to connect to database");

    let mods: std::collections::HashMap<String, ModData> =
        serde_json::from_str(include_str!("./mods.json")).expect("valid seed_mods.json");

    for m in mods.values() {
        println!("{m:?}");

        let game = entity::games::Entity::find()
            .filter(entity::games::Column::Slug.eq(m.game.to_lowercase()))
            .one(&db)
            .await
            .expect("query to succeed")
            .expect("game to exist");

        // If a mod with this slug is found, delete it
        entity::mods::Entity::delete_many()
            .filter(entity::mods::Column::Slug.eq(m.internalName.clone()))
            .exec(&db)
            .await
            .expect("mod delete to succeed");

        let mod_active = entity::mods::ActiveModel {
            id: Set(Uuid::new_v4()),
            slug: Set(m.internalName.clone()),
            name: Set(m.displayName.clone()),
            description: Set(m.description.clone()),
            game_id: Set(game.id),
            published_at: Set(Some(parse_date(&m.releaseDate))),
            ..Default::default()
        };

        let inserted_mod = mod_active.insert(&db).await.expect("mod insert to succeed");

        let release_active = entity::mod_releases::ActiveModel {
            id: Set(Uuid::new_v4()),
            mod_id: Set(inserted_mod.id),
            version: Set(m.version.clone()),
            description: Set(m.description.clone()),
            download_url: Set(m.downloadUrl.clone()),
            created_at: Set(parse_date(&m.releaseDate)),
            ..Default::default()
        };

        release_active.insert(&db).await.expect("release insert to succeed");
    }
}
