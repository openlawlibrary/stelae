use sea_orm::Schema;
use sea_orm_migration::prelude::*;

pub mod publication {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    use crate::m00001_create_repository::repository;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
    #[sea_orm(table_name = "publication")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub date: Date,
        pub revoked: bool,
        pub repository_id: i32,
        pub core_version: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter)]
    pub enum Relation {
        Repository,
    }

    impl RelationTrait for Relation {
        fn def(&self) -> RelationDef {
            match self {
                Self::Repository => Entity::belongs_to(repository::Entity)
                    .from(Column::RepositoryId)
                    .to(repository::Column::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .into(),
            }
        }
    }

    impl Related<repository::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Repository.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m00002_create_publication"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let builder = manager.get_database_backend();
        let schema = Schema::new(builder);
        manager
            .create_table(schema.create_table_from_entity(publication::Entity))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(publication::Entity).to_owned())
            .await
    }
}
