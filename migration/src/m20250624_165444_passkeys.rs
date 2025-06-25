use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Passkeys::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Passkeys::Id).binary().not_null().primary_key())
                    .col(ColumnDef::new(Passkeys::UserId).uuid().not_null())
                    .col(ColumnDef::new(Passkeys::Data).json_binary().not_null()) // Opaque webauthn_rs::prelude::Passkey
                    .col(ColumnDef::new(Passkeys::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Passkeys::LastUsedAt).timestamp_with_time_zone().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Passkeys::Table, Passkeys::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Passkeys::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Passkeys {
    Table,
    Id,
    UserId,
    Data,
    CreatedAt,
    LastUsedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
