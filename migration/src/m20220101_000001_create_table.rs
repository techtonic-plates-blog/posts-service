use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    schema::*,
    sea_orm::{EnumIter, Iterable},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(PostsStatusEnum)
                    .values(PostStatusVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(uuid(Users::Id).primary_key())
                    .to_owned(),
            )
            .await?;

        manager.create_table(Table::create().table(Tags::Table)
            .if_not_exists()
            .col(uuid(Tags::Id).primary_key())
            .col(string(Tags::Name).not_null().unique_key())
            .to_owned()).await?;

        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(uuid(Posts::Id).primary_key().not_null())
                    .col(string(Posts::Slug).not_null().unique_key())
                    .col(string(Posts::Title).not_null())
                    .col(string_null(Posts::TitleImageUrl))
                    .col(date_time(Posts::CreationTime).default(Expr::current_timestamp()))
                    .col(string(Posts::Body).not_null())
                    .col(string(Posts::Author).not_null())
                    .col(uuid(Posts::CreatedBy).not_null())
                    .col(string(Posts::Subheading).not_null())
                    .col(date_time_null(Posts::LastEdit))
                    .col(
                        enumeration(
                            Posts::PostStatus,
                            PostsStatusEnum,
                            PostStatusVariants::iter(),
                        )
                        .default(Expr::value(PostStatusVariants::Draft.to_string())),
                    )
                    .col(
                        custom(Posts::TitleSearch, "TSVECTOR")
                            .extra("GENERATED ALWAYS AS (to_tsvector('english',title)) STORED"),
                    )
                    .col(
                        custom(Posts::AuthorSearch, "TSVECTOR")
                            .extra("GENERATED ALWAYS AS (to_tsvector('english',author)) STORED"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::CreatedBy)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;
        
        manager
            .create_table(
                Table::create()
                    .table(PostTags::Table)
                    .if_not_exists()
                    .col(uuid(PostTags::PostId).not_null())
                    .col(uuid(PostTags::TagId).not_null())
                    .primary_key(
                        Index::create()
                            .name("pk_post_tags")
                            .col(PostTags::PostId)
                            .col(PostTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_tags_post_id")
                            .from(PostTags::Table, PostTags::PostId)
                            .to(Posts::Table, Posts::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_tags_tag_id")
                            .from(PostTags::Table, PostTags::TagId)
                            .to(Tags::Table, Tags::Id),
                    )
                    .to_owned(),
            )
            .await?;


        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
struct PostsStatusEnum;

#[derive(DeriveIden, EnumIter)]
enum PostStatusVariants {
    Draft,
    Published,
    Archived,
    Removed,
}

#[derive(DeriveIden)]
enum Posts {
    Id,
    Table,
    Slug,
    Title,
    TitleImageUrl,
    Subheading,
    Author,
    CreationTime,
    LastEdit,
    Body,
    CreatedBy,
    PostStatus,
    #[sea_orm(ignore)]
    TitleSearch,

    #[sea_orm(ignore)]
    AuthorSearch,
}

#[derive(DeriveIden)]
enum Users {
    Id,
    Table,
}

#[derive(DeriveIden)]
enum PostTags {
    Table,
    PostId,
    TagId
}

#[derive(DeriveIden)]
enum Tags {
    Table,
    Id,
    Name
}