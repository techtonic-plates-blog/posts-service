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
                    .col(uuid(Posts::Id).primary_key().not_null())
                    .col(string(Posts::Slug).not_null().unique_key())
                    .col(string(Posts::Title))
                    .col(date_time(Posts::CreationTime).default(Expr::current_timestamp()))
          
                    .col(string(Posts::TypstFile))
                    .col(string(Posts::Author))
                    .col(custom(Posts::TitleSearch, "TSVECTOR").extra("GENERATED ALWAYS AS (to_tsvector('english',title)) STORED"))
                     .col(custom(Posts::AuthorSearch, "TSVECTOR").extra("GENERATED ALWAYS AS (to_tsvector('english',author)) STORED"))
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
    Id,
    Table,
    Slug,
    Title,
    Author,
    CreationTime,
    TypstFile,
    #[sea_orm(ignore)]
    TitleSearch,

    #[sea_orm(ignore)]
    AuthorSearch
}
