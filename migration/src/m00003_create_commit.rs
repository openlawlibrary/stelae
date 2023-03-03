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
    #[derive(Copy, Clone, Debug, EnumIter)]
    pub enum Relation {
        Publication,
    }

    impl RelationTrait for Relation {
        fn def(&self) -> RelationDef {
            match self {
                Self::Publication => Entity::belongs_to(publication::Entity)
                    .from(Column::PublicationId)
                    .to(publication::Column::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .into(),
            }
        }
    }

    impl Related<publication::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Publication.def()
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
            .create_table(schema.create_table_from_entity(commit::Entity))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
        .drop_table(Table::drop().table(commit::Entity).to_owned())
        .await
    }
}

