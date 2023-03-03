pub use sea_orm_migration::prelude::*;

pub mod m00001_create_repository;
pub mod m00002_create_publication;
pub mod m00003_create_commit;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_repository::Migration),
            Box::new(m00002_create_publication::Migration),
            Box::new(m00003_create_commit::Migration),
        ]
    }
}
