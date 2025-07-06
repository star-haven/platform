use sea_orm_migration::{prelude::{extension::postgres::TypeDropStatement, *}, schema::*, sea_orm::{ActiveEnum, DbBackend, DeriveActiveEnum, EnumIter, Schema}};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);

        manager.alter_table(
            Table::alter()
                .table(Mods::Table)
                .add_column(string_null(Mods::ThumbnailUrl))
                .to_owned()
        ).await?;
        manager.create_type(schema.create_enum_from_active_enum::<ModMediaType>()).await?;
        manager
            .create_table(
                Table::create()
                    .table(ModMedia::Table)
                    .if_not_exists()
                    .col(pk_uuid(ModMedia::Id))
                    .col(uuid(ModMedia::ModId))
                    .col(custom(ModMedia::MediaType, ModMediaType::name()))
                    .col(string(ModMedia::Url))
                    .col(integer(ModMedia::Position))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ModMedia::Table, ModMedia::ModId)
                            .to(Mods::Table, Mods::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_mod_media_mod_id_position")
                    .table(ModMedia::Table)
                    .col(ModMedia::ModId)
                    .col(ModMedia::Position)
                    .unique()
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_mod_media_mod_id_position")
                    .table(ModMedia::Table)
                    .to_owned()
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ModMedia::Table)
                    .to_owned()
            )
            .await?;
        manager
            .drop_type(
                TypeDropStatement::new()
                    .name(ModMediaType::name())
                    .to_owned()
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Mods::Table)
                    .drop_column(Mods::ThumbnailUrl)
                    .to_owned()
            )
            .await
    }
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum Mods {
    Table,
    Id,
    Slug,
    Name,
    Description,
    GameId,
    PublishedAt,
    ThumbnailUrl,
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum ModMedia {
    Table,
    Id,
    ModId,
    MediaType,
    Url,
    Position,
}

#[derive(EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mod_media_type")]
pub enum ModMediaType {
    #[sea_orm(string_value = "image")]
    Image,
    #[sea_orm(string_value = "youtube")]
    Youtube,
}
