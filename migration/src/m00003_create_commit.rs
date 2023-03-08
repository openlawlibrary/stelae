use sea_orm::Schema;
use sea_orm_migration::prelude::*;

pub mod commit {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    use crate::m00002_create_publication::publication;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
    #[sea_orm(table_name = "commit")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub sha: String,
        pub date: Date,
        pub revoked: bool,
        pub publication_id: i32,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "publication::Entity",
            from = "Column::PublicationId",
            to = "publication::Column::Id",
            on_update = "Cascade",
            on_delete = "Cascade"
        )]
        Publication,
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
            .create_table(schema.create_table_from_entity(commit::Entity))
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(commit::Entity)
                    .name("commit__unique__publication_id__sha")
                    .unique()
                    .col(commit::Column::PublicationId)
                    .col(commit::Column::Sha)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(commit::Entity)
                    .name("commit__date__id")
                    .col(commit::Column::Date)
                    .col(commit::Column::Id)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(commit::Entity)
                    .name("commit__publication_id__date")
                    .col(commit::Column::PublicationId)
                    .col(commit::Column::Date)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
        .drop_table(Table::drop().table(commit::Entity).to_owned())
        .await
    }
}

