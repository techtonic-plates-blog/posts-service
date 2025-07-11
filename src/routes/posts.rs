use std::str::FromStr;

use crate::auth::BearerAuthorization;
use crate::routes::ApiTags;

use entities::posts::Entity as Posts;
use migration::Expr;
use poem::error::BadRequest;
use poem::Error;
use poem::{Result, error::InternalServerError, web::Data};
use poem_openapi::param::Query;
use poem_openapi::payload::PlainText;
use poem_openapi::{ApiResponse, OpenApi, param::Path, payload::Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use sea_orm::{DatabaseConnection, QueryOrder, Value};
use uuid::Uuid;

pub struct PostsApi;
#[derive(ApiResponse)]
enum GetPostResponse {
    #[oai(status = 200)]
    Ok(Json<entities::posts::Model>),
    #[oai(status = 404)]
    NotFound,
}

#[derive(serde::Serialize, poem_openapi::Object)]
pub struct GetPostsData {
    pub posts: Vec<entities::posts::Model>,
    pub count: Option<u64>,
}

#[derive(poem_openapi::Object)]
struct InsertPostRequest {
    pub title: String,
    pub author: String,
    pub body: String,
    pub subheading: String,
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
            Some(post) => return Ok(GetPostResponse::Ok(Json(post))),
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
    ) -> Result<Json<GetPostsData>> {
        use sea_orm::{ColumnTrait, QueryFilter, QuerySelect};
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
        Ok(Json(GetPostsData { posts, count }))
    }

    #[oai(method = "post", path = "/bulk_get")]
    async fn bulk_get(
        &self,
        db: Data<&DatabaseConnection>,
        slugs: Json<Vec<String>>,
    ) -> Result<Json<Vec<entities::posts::Model>>> {
        if slugs.0.is_empty() {
            Ok(Json(vec![]))
        } else {
            Ok(Json(
                Posts::find()
                    .filter(entities::posts::Column::Slug.is_in(slugs.0.clone()))
                    .all(*db)
                    .await
                    .map_err(InternalServerError)?,
            ))
        }
    }

    #[oai(path = "/", method = "post")]
    async fn insert_post(&self, db: Data<&DatabaseConnection>, claims: BearerAuthorization, request: Json<InsertPostRequest>) -> Result<PlainText<String>> {
        if !claims.permissions.contains(&"create post".to_string()) {
            return Err(Error::from_string(
                "Not enough permissions",
                poem::http::StatusCode::UNAUTHORIZED,
            ));
        }

        let user_id = &claims.sub;

        let user = entities::users::Entity::find_by_id(uuid::Uuid::from_str(user_id).map_err(BadRequest)?)
            .one(*db)
            .await
            .map_err(InternalServerError)?
            .ok_or_else(|| Error::from_string("User not found", poem::http::StatusCode::UNAUTHORIZED))?;


        let new_post = entities::posts::ActiveModel {
            id: Set(Uuid::new_v4()),
            slug: Set(request.title.to_lowercase().replace(" ", "-")),
            title: Set(request.title.clone()),
            body: Set(request.body.clone()),
            created_by: Set(user.id),

            ..Default::default()
        };

        let post = new_post.insert(*db).await.map_err(InternalServerError)?;
        Ok(PlainText(format!("/posts/{}", post.slug)))
    }

   /* #[oai(method = "put", path = "/:post_slug")]
    async fn put_post(
        &self,
        post_slug: Path<String>,
        claims: BearerAuthorization,
        db: Data<&DatabaseConnection>,
        object_storage: Data<&ObjectStorage>,
        request: PutPostRequest,
    ) -> Result<PlainText<String>> {

       if !claims.permissions.contains(&"create post".to_string()) {
             return Err(Error::from_string(
                "Not enough permissions",
                poem::http::StatusCode::UNAUTHORIZED,
            ));
        }

        let now = Utc::now().naive_utc();
        let existing = Entity::find()
            .filter(Column::Slug.eq(post_slug.0.clone()))
            .one(*db)
            .await
            .map_err(InternalServerError)?;

        let post = if let Some(mut model) = existing {
            let file_data = request.typst_file.into_vec().await.map_err(BadRequest)?;

            let file_name = format!("{}.typ", model.slug);

            let obj = object_storage.put_object_content(
                TYPST_FILES_BUCKET.to_string(),
                file_name,
                Bytes::from(file_data),
            );
            obj.send().await.unwrap();

            model.title = request.title.clone();

            model.author = request.author.clone();

            model.creation_time = model.creation_time;
            let active: ActiveModel = model.into();
            active.update(*db).await.map_err(InternalServerError)?
        } else {
            let file_data = request.typst_file.into_vec().await.map_err(BadRequest)?;
            let file_name = format!("{}.typ", post_slug.0);

            let obj = object_storage.put_object_content(
                TYPST_FILES_BUCKET.to_string(),
                &file_name,
                Bytes::from(file_data),
            );
            obj.send().await.map_err(InternalServerError)?;
            let new = ActiveModel {
                id: Set(Uuid::new_v4()),
                slug: Set(post_slug.0.clone()),
                title: Set(request.title.clone()),
                typst_file: Set(file_name),
                author: Set(request.author.clone()),

                creation_time: Set(now),
                created_by: Set(claims.id)
                ..Default::default()
            };
            new.insert(*db).await.map_err(InternalServerError)?
        };
        Ok(PlainText(format!("/posts/{}", post.slug)))
    }*/
}
