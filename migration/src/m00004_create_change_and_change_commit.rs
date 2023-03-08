use sea_orm::Schema;
use sea_orm_migration::prelude::*;

//TODO: explanation as to why two modules are needed
pub mod change {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
    #[sea_orm(rs_type = "String", db_type = "String(Some(100))")]
    pub enum Status {
        #[sea_orm(string_value = "ELEMENT_CHANGED")]
        ElementChanged,
        #[sea_orm(string_value = "SUBELEMENTS_CHANGED")]
        SubelementsChanged,
        #[sea_orm(string_value = "ELEMENT_ADDED")]
        ElementAdded,
        #[sea_orm(string_value = "ELEMENT_REMOVED")]
        ElementRemoved
    }


    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
    #[sea_orm(table_name = "change")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub codified_date: Date,
        pub reason: Option<String>,
        pub url: String,
        pub status: Status,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::change_commit::Entity")]
        ChangeCommit,
    }
    
    impl Related<super::change_commit::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::ChangeCommit.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod change_commit {
    use sea_orm::entity::prelude::*;

    use crate::m00003_create_commit::commit;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "change_commit")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub change_id: i32,
        pub commit_id: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::change::Entity",
            from = "Column::ChangeId",
            to = "super::change::Column::Id",
            on_update = "Cascade",
            on_delete = "Cascade"
        )]
        Change,
        #[sea_orm(
            belongs_to = "commit::Entity",
            from = "Column::CommitId",
            to = "commit::Column::Id",
            on_update = "Cascade",
            on_delete = "Cascade"
        )]
        Commit,
    }

    impl Related<super::change::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Change.def()
        }
    }

    impl Related<commit::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Commit.def()
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
            .create_table(schema.create_table_from_entity(change::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(change_commit::Entity))
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(change_commit::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(change::Entity).to_owned())
            .await?;
        Ok(())
    }
}
