use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
  

        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Posts::Id).string().not_null().primary_key())
                    .col(string(Posts::Title))
                    .col(date_time(Posts::CreationTime))
                    .col(date_time_null(Posts::PostTime))
                    .col(string_null(Posts::PostUrl))
                    .col(string(Posts::TypstFile))
                    .col(string(Posts::Author))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Posts {
    Table,
    Id,
    Title,
    Author,
    CreationTime,
    PostTime,
    TypstFile,
    PostUrl
}
