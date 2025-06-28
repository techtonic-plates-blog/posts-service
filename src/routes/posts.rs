
use crate::connections::ObjectStorage;
use crate::connections::object_storage::TYPST_FILES_BUCKET;
use crate::routes::ApiTags;
use bytes::Bytes;
use chrono::Utc;
use entities::posts::Entity as Posts;
use entities::posts::{ActiveModel, Column, Entity};
use migration::Expr;
use poem::error::BadRequest;
use poem::{Result, error::InternalServerError, web::Data};
use poem_openapi::payload::PlainText;
use poem_openapi::Multipart;
use poem_openapi::param::Query;
use poem_openapi::types::multipart::Upload;
use poem_openapi::{ApiResponse, OpenApi, param::Path, payload::Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use sea_orm::{DatabaseConnection, PaginatorTrait, QueryOrder, Value};
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

#[derive(Multipart, Debug)]
pub struct PutPostRequest {
    pub title: String,
    pub typst_file: Upload,
    pub author: String,
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

    #[oai(method = "put", path = "/:post_slug")]
    async fn put_post(
        &self,
        post_slug: Path<String>,
        db: Data<&DatabaseConnection>,
        object_storage: Data<&ObjectStorage>,
        request: PutPostRequest,
    ) -> Result<PlainText<String>> {
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
            obj.send().await.unwrap();
            let new = ActiveModel {
                id: Set(Uuid::new_v4()),
                slug: Set(post_slug.0.clone()),
                title: Set(request.title.clone()),
                typst_file: Set(file_name),
                author: Set(request.author.clone()),
              
                creation_time: Set(now),
                ..Default::default()
            };
            new.insert(*db).await.map_err(InternalServerError)?
        };
        Ok(PlainText(format!("/posts/{}", post.slug)))
    }
}
