use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(Games::Table)
                .add_column(string_uniq(Games::Slug))
                .to_owned()
        ).await?;
        manager.get_connection().execute_unprepared(
            "INSERT INTO games (id, name, console_name, year, slug) VALUES
                (gen_random_uuid(), 'Paper Mario', 'Nintendo 64', 2000, 'pm64'),
                (gen_random_uuid(), 'The Thousand-Year Door', 'GameCube', 2004, 'ttyd'),
                (gen_random_uuid(), 'Super Paper Mario', 'Wii', 2007, 'spm'),
                (gen_random_uuid(), 'Sticker Star', 'Nintendo 3DS', 2012, 'ss'),
                (gen_random_uuid(), 'Color Splash', 'Wii U', 2016, 'cs'),
                (gen_random_uuid(), 'The Origami King', 'Nintendo Switch', 2020, 'tok'),
                (gen_random_uuid(), 'The Thousand-Year Door (Remake)', 'Nintendo Switch', 2024, 'ttyd-switch');"
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "DELETE FROM games WHERE slug IN (
                'pm64', 'ttyd', 'spm', 'ss', 'cs', 'tok', 'ttyd-switch'
            );"
        ).await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Games::Table)
                    .drop_column(Games::Slug)
                    .to_owned()
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum Games {
    Table,
    Id,
    Name,
    ConsoleName,
    Year,
    Slug,
}
