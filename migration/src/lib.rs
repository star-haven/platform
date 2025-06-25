pub use sea_orm_migration::prelude::*;

mod m20250623_153441_create_users;
mod m20250624_165444_passkeys;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250623_153441_create_users::Migration),
            Box::new(m20250624_165444_passkeys::Migration),
        ]
    }
}
