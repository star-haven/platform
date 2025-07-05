use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Games::Table)
                    .if_not_exists()
                    .col(pk_uuid(Games::Id))
                    .col(string(Games::Name))
                    .col(string(Games::ConsoleName))
                    .col(integer(Games::Year))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Mods::Table)
                    .if_not_exists()
                    .col(pk_uuid(Mods::Id))
                    .col(string_uniq(Mods::Slug))
                    .col(string(Mods::Name))
                    .col(string(Mods::Description))
                    .col(uuid(Mods::GameId))
                    .col(timestamp_with_time_zone_null(Mods::PublishedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Mods::Table, Mods::GameId)
                            .to(Games::Table, Games::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_mods_name_fulltext ON mods USING GIN(to_tsvector('english', name));"
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(ModAuthors::Table)
                    .if_not_exists()
                    .col(pk_uuid(ModAuthors::Id))
                    .col(uuid(ModAuthors::ModId))
                    .col(uuid(ModAuthors::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ModAuthors::Table, ModAuthors::ModId)
                            .to(Mods::Table, Mods::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ModAuthors::Table, ModAuthors::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(ModReleases::Table)
                    .if_not_exists()
                    .col(pk_uuid(ModReleases::Id))
                    .col(uuid(ModReleases::ModId))
                    .col(string(ModReleases::Version))
                    .col(string(ModReleases::Description))
                    .col(string(ModReleases::DownloadUrl))
                    .col(timestamp_with_time_zone(ModReleases::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ModReleases::Table, ModReleases::ModId)
                            .to(Mods::Table, Mods::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_modreleases_created_at")
                    .table(ModReleases::Table)
                    .col(ModReleases::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_index(Index::drop().name("idx_mods_name_fulltext").to_owned()).await?;
        manager.drop_index(Index::drop().name("idx_modreleases_created_at").to_owned()).await?;
        manager.drop_table(Table::drop().table(ModReleases::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(ModAuthors::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Mods::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Mods {
    Table,
    Id,
    Slug,
    Name,
    Description,
    GameId,
    PublishedAt,
}

#[derive(DeriveIden)]
enum ModAuthors {
    Table,
    Id,
    ModId,
    UserId,
}

#[derive(DeriveIden)]
enum ModReleases {
    Table,
    Id,
    ModId,
    Version,
    Description,
    DownloadUrl,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Games {
    Table,
    Id,
    Name,
    ConsoleName,
    Year,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
