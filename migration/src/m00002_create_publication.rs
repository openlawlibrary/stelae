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

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "repository::Entity",
            from = "Column::RepositoryId",
            to = "repository::Column::Id",
            on_update = "Cascade",
            on_delete = "Cascade"
        )]
        Repository,
    }

    impl Related<repository::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Repository.def()
        }
    }
    impl ActiveModelBehavior for ActiveModel {}
}
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let builder = manager.get_database_backend();
        let schema = Schema::new(builder);
        manager
            .create_table(schema.create_table_from_entity(publication::Entity))
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(publication::Entity)
                    .name("publication__unique__repository_id__name")
                    .unique()
                    .col(publication::Column::RepositoryId)
                    .col(publication::Column::Name)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(publication::Entity).to_owned())
            .await
    }
}
