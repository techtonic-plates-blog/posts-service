use std::str::FromStr;

use crate::auth::BearerAuthorization;
use crate::routes::ApiTags;

use entities::posts::Entity as Posts;
use entities::sea_orm_active_enums::PostsStatusEnum;
use migration::Expr;
use poem::Error;
use poem::error::BadRequest;
use poem::{Result, error::InternalServerError, web::Data};
use poem_openapi::param::Query;
use poem_openapi::payload::PlainText;
use poem_openapi::{ApiResponse, OpenApi, param::Path, payload::Json};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, Set, TransactionTrait, QuerySelect,
};
use sea_orm::{DatabaseConnection, QueryOrder, Value};
use uuid::Uuid;

pub struct PostsApi;

#[derive(serde::Serialize, poem_openapi::Object)]
pub struct PostWithTags {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub hero_image: Option<String>,
    pub creation_time: chrono::NaiveDateTime,
    pub body: String,
    pub author: String,
    pub created_by: Uuid,
    pub subheading: String,
    pub last_edit: Option<chrono::NaiveDateTime>,
    pub post_status: PostsStatusEnum,
    pub tags: Vec<String>,
}

#[derive(ApiResponse)]
enum GetPostResponse {
    #[oai(status = 200)]
    Ok(Json<PostWithTags>),
    #[oai(status = 404)]
    NotFound,
}

#[derive(serde::Serialize, poem_openapi::Object)]
pub struct GetPostsResponse {
    pub posts: Vec<String>,
    pub count: Option<u64>,
}

#[derive(poem_openapi::Object)]
struct InsertPostRequest {
    #[oai(validator(min_length = 10))]
    pub title: String,
    #[oai(validator(min_length = 3))]
    pub author: String,
    pub body: String,
    #[oai(validator(min_length = 3))]
    pub subheading: String,
    pub tags: Option<Vec<String>>,
    #[oai(validator(min_length = 5))]
    pub hero_image: Option<String>,
}

#[derive(ApiResponse)]
enum InsertPostResponse {
    #[oai(status = 201)]
    Created(PlainText<String>),
    #[oai(status = 409)]
    Conflict,
}

#[derive(poem_openapi::Object)]
struct PatchPostRequest {
    #[oai(validator(min_length = 10))]
    pub title: Option<String>,
    #[oai(validator(min_length = 3))]
    pub author: Option<String>,
    pub body: Option<String>,
    #[oai(validator(min_length = 3))]
    pub subheading: Option<String>,
    pub status: Option<PostsStatusEnum>,
    pub tags: Option<Vec<String>>,
    #[oai(validator(min_length = 5))]
    pub hero_image: Option<String>,
}

#[derive(ApiResponse)]
enum PatchPostResponse {
    #[oai(status = 200)]
    Ok(PlainText<String>),
    #[oai(status = 404)]
    NotFound,
    #[oai(status = 409)]
    Conflict,
}

#[derive(ApiResponse)]
enum DeletePostResponse {
    #[oai(status = 200)]
    Ok(PlainText<String>),
    #[oai(status = 404)]
    NotFound,
}

async fn create_tag_if_not_exists<T: ConnectionTrait>(
    db: &T,
    tag_name: &str,
) -> Result<entities::tags::Model, Error> {
    let tag = entities::tags::Entity::find()
        .filter(entities::tags::Column::Name.eq(tag_name))
        .one(db)
        .await
        .map_err(InternalServerError)?;

    if let Some(tag) = tag {
        return Ok(tag);
    }

    let tag = entities::tags::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(tag_name.to_string()),
    };
    tag.insert(db).await.map_err(InternalServerError)
}

#[OpenApi(prefix_path = "/posts", tag = "ApiTags::Posts")]
impl PostsApi {
    #[oai(method = "get", path = "/:post_slug")]
    async fn get_post(
        &self,
        post_slug: Path<String>,
        db: Data<&DatabaseConnection>,
    ) -> Result<GetPostResponse> {
        let post: Option<entities::posts::Model> = Posts::find()
            .filter(entities::posts::Column::Slug.eq(post_slug.0))
            .one(*db)
            .await
            .map_err(InternalServerError)?;

        match post {
            Some(post) => {
                // Fetch tags for this post
                let tags: Vec<entities::tags::Model> = post
                    .find_related(entities::tags::Entity)
                    .all(*db)
                    .await
                    .map_err(InternalServerError)?;

                let tag_names: Vec<String> = tags.into_iter().map(|tag| tag.name).collect();

                let post_with_tags = PostWithTags {
                    id: post.id,
                    slug: post.slug,
                    title: post.title,
                    hero_image: post.hero_image,
                    creation_time: post.creation_time,
                    body: post.body,
                    author: post.author,
                    created_by: post.created_by,
                    subheading: post.subheading,
                    last_edit: post.last_edit,
                    post_status: post.post_status,
                    tags: tag_names,
                };

                return Ok(GetPostResponse::Ok(Json(post_with_tags)));
            }
            None => return Ok(GetPostResponse::NotFound),
        }
    }

    #[oai(method = "get", path = "/")]
    async fn get_posts(
        &self,
        db: Data<&DatabaseConnection>,
        limit: Query<Option<u64>>,
        offset: Query<Option<u64>>,
        title: Query<Option<String>>,
        author: Query<Option<String>>,
        add_count: Query<Option<bool>>,
        creation_time: Query<Option<String>>,
        status: Query<Option<PostsStatusEnum>>,
    ) -> Result<Json<GetPostsResponse>> {
        let limit = limit.0.unwrap_or(20);
        let offset = offset.0.unwrap_or(0);
        let mut query = Posts::find();
        if let Some(title) = &title.0 {
            query = query.filter(Expr::cust_with_values(
                "title_search @@ to_tsquery($1)",
                vec![Value::String(Some(Box::new(title.clone())))],
            ));
        }
        if let Some(author) = &author.0 {
            query = query.filter(Expr::cust_with_values(
                "author_search @@ to_tsquery($1)",
                vec![Value::String(Some(Box::new(author.clone())))],
            ));
        }
        if let Some(creation_time) = &creation_time.0 {
            query = query.filter(entities::posts::Column::CreationTime.gte(creation_time));
        }
        if let Some(status) = &status.0 {
            query = query.filter(entities::posts::Column::PostStatus.eq(status.clone()));
        }
        let posts: Vec<entities::posts::Model> = query
            .clone()
            .order_by(entities::posts::Column::CreationTime, sea_orm::Order::Desc)
            .limit(limit)
            .offset(offset)
            .all(*db)
            .await
            .map_err(InternalServerError)?;
        let mut count = None;
        if add_count.0.unwrap_or(false) {
            let c = query.count(*db).await.map_err(InternalServerError)?;
            count = Some(c);
        }
        let ids = posts
            .iter()
            .map(|post| post.slug.clone())
            .collect::<Vec<String>>();

        Ok(Json(GetPostsResponse { 
            posts: ids, 
            count
        }))
    }

    #[oai(method = "post", path = "/bulk_get")]
    async fn bulk_get(
        &self,
        db: Data<&DatabaseConnection>,
        slugs: Json<Vec<String>>,
    ) -> Result<Json<Vec<PostWithTags>>> {
        if slugs.0.is_empty() {
            Ok(Json(vec![]))
        } else {
            let posts: Vec<entities::posts::Model> = Posts::find()
                .filter(entities::posts::Column::Slug.is_in(slugs.0.clone()))
                .all(*db)
                .await
                .map_err(InternalServerError)?;

            let mut posts_with_tags = Vec::new();
            
            for post in posts {
                // Fetch tags for each post
                let tags: Vec<entities::tags::Model> = post
                    .find_related(entities::tags::Entity)
                    .all(*db)
                    .await
                    .map_err(InternalServerError)?;

                let tag_names: Vec<String> = tags.into_iter().map(|tag| tag.name).collect();

                let post_with_tags = PostWithTags {
                    id: post.id,
                    slug: post.slug,
                    title: post.title,
                    hero_image: post.hero_image,
                    creation_time: post.creation_time,
                    body: post.body,
                    author: post.author,
                    created_by: post.created_by,
                    subheading: post.subheading,
                    last_edit: post.last_edit,
                    post_status: post.post_status,
                    tags: tag_names,
                };

                posts_with_tags.push(post_with_tags);
            }

            Ok(Json(posts_with_tags))
        }
    }

    #[oai(path = "/", method = "post")]
    async fn insert_post(
        &self,
        db: Data<&DatabaseConnection>,
        claims: BearerAuthorization,
        request: Json<InsertPostRequest>,
    ) -> Result<InsertPostResponse> {
        if !claims.has_permission("create", "post") {
            return Err(Error::from_string(
                "Not enough permissions",
                poem::http::StatusCode::FORBIDDEN
            ));
        }

        let user_id = &claims.sub;

        let user =
            entities::users::Entity::find_by_id(uuid::Uuid::from_str(user_id).map_err(BadRequest)?)
                .one(*db)
                .await
                .map_err(InternalServerError)?
                .ok_or_else(|| {
                    Error::from_string("User not found", poem::http::StatusCode::UNAUTHORIZED)
                })?;

        let post_exists = Posts::find()
            .filter(
                entities::posts::Column::Slug.eq(request.title.to_lowercase().replace(" ", "_")),
            )
            .one(*db)
            .await
            .map_err(InternalServerError)?;

        if post_exists.is_some() {
            return Ok(InsertPostResponse::Conflict);
        }

        let tags = request.tags.clone().unwrap_or_default();

        let tnx = db.begin().await.map_err(InternalServerError)?;

        let mut tag_ids = Vec::new();
        for tag in tags {
            let tag = create_tag_if_not_exists(&tnx, &tag).await?;
            tag_ids.push(tag.id);
        }

        let new_post = entities::posts::ActiveModel {
            id: Set(Uuid::new_v4()),
            slug: Set(request.title.to_lowercase().replace(" ", "_")),
            title: Set(request.title.clone()),
            body: Set(request.body.clone()),
            created_by: Set(user.id),
            author: Set(request.author.clone()),
            subheading: Set(request.subheading.clone()),
            creation_time: Set(chrono::Utc::now().naive_utc()),
            last_edit: Set(None),
            hero_image: Set(request.hero_image.clone()),
            ..Default::default()
        };

        let post = new_post.insert(&tnx).await.map_err(InternalServerError)?;

        for tag_id in tag_ids {
            let post_tag = entities::post_tags::ActiveModel {
                post_id: Set(post.id),
                tag_id: Set(tag_id),
            };
            post_tag.insert(&tnx).await.map_err(InternalServerError)?;
        }

        tnx.commit().await.map_err(InternalServerError)?;

        Ok(InsertPostResponse::Created(PlainText(format!(
            "/posts/{}",
            post.slug
        ))))
    }

    #[oai(method = "delete", path = "/:post_slug")]
    async fn delete_post(
        &self,
        post_slug: Path<String>,
        claims: BearerAuthorization,
        db: Data<&DatabaseConnection>,
    ) -> Result<DeletePostResponse> {
        if !claims.has_permission("delete", "post") {
            return Err(Error::from_string(
                "Not enough permissions",
                poem::http::StatusCode::FORBIDDEN,
            ));
        }
        let post = Posts::find()
            .filter(entities::posts::Column::Slug.eq(&post_slug.0))
            .one(*db)
            .await
            .map_err(InternalServerError)?;

        let post = match post {
            Some(post) => post,
            None => return Ok(DeletePostResponse::NotFound),
        };

        post.delete(*db).await.map_err(InternalServerError)?;
        Ok(DeletePostResponse::Ok(PlainText(format!(
            "Post {} deleted",
            post_slug.0
        ))))
    }

    #[oai(method = "patch", path = "/:post_slug")]
    async fn patch_post(
        &self,
        post_slug: Path<String>,
        claims: BearerAuthorization,
        db: Data<&DatabaseConnection>,
        request: Json<PatchPostRequest>,
    ) -> Result<PatchPostResponse> {
        if !claims.has_permission("update", "post") {
            return Err(Error::from_string(
                "Not enough permissions",
                poem::http::StatusCode::FORBIDDEN,
            ));
        }

        let post_model = Posts::find()
            .filter(entities::posts::Column::Slug.eq(post_slug.0.clone()))
            .one(*db)
            .await
            .map_err(InternalServerError)?;

        let mut post: entities::posts::ActiveModel = match post_model {
            Some(model) => model.into(),
            None => return Ok(PatchPostResponse::NotFound),
        };

        let tnx = db.begin().await.map_err(InternalServerError)?;

        if let Some(title) = &request.title {
            post.title = Set(title.clone());
            let slug = title.to_lowercase().replace(" ", "_");

            let slug_being_used = Posts::find()
                .filter(entities::posts::Column::Slug.eq(slug.clone()))
                .one(&tnx)
                .await
                .map_err(InternalServerError)?;
            if let Some(existing_post) = slug_being_used {
                let current_id = post.id.as_ref();
                if existing_post.id != *current_id {
                    return Ok(PatchPostResponse::Conflict);
                }
            }

            post.slug = Set(slug);
        }
        if let Some(author) = &request.author {
            post.author = Set(author.clone());
        }
        if let Some(body) = &request.body {
            post.body = Set(body.clone());
        }
        if let Some(subheading) = &request.subheading {
            post.subheading = Set(subheading.clone());
        }
        if let Some(status) = &request.status {
            post.post_status = Set(status.clone());
        }
        if let Some(hero_image) = &request.hero_image {
            post.hero_image = Set(Some(hero_image.clone()));
        }

        if let Some(tags) = &request.tags {
            let current_id = post.id.as_ref();
            entities::post_tags::Entity::delete_many()
                .filter(entities::post_tags::Column::PostId.eq(current_id.clone()))
                .exec(&tnx)
                .await
                .map_err(InternalServerError)?;

            let mut tag_ids = Vec::new();
            for tag in tags {
                let tag = create_tag_if_not_exists(&tnx, tag).await?;
                tag_ids.push(tag.id);
            }

            for tag_id in tag_ids {
                let post_tag = entities::post_tags::ActiveModel {
                    post_id: post.id.clone(),
                    tag_id: Set(tag_id),
                };
                post_tag.insert(&tnx).await.map_err(InternalServerError)?;
            }
        }
        #[allow(unused)]
        let mut slug = post_slug.0;
        if post.is_changed() {
            post.last_edit = Set(Some(chrono::Utc::now().naive_utc()));
            let model = post.update(&tnx).await.map_err(InternalServerError)?;
            slug = model.slug;
        } else {
            return Ok(PatchPostResponse::Ok(PlainText(
                "No changes made".to_string(),
            )));
        }

        tnx.commit().await.map_err(InternalServerError)?;

        Ok(PatchPostResponse::Ok(PlainText(format!("{}", slug))))
    }
}
