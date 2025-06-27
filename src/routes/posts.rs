use poem::{error::InternalServerError, web::Data, Result};
use poem_openapi::{param::Path, payload::Json, ApiResponse, OpenApi};
use sea_orm::{DatabaseConnection, EntityTrait};
use entities::posts::Entity as Posts;

pub struct PostsApi;
#[derive(ApiResponse)]
enum GetPostResponse {
    #[oai(status = 200)]
    Ok(Json<entities::posts::Model>),
    #[oai(status = 404)]
    NotFound
}

#[OpenApi(prefix_path = "/posts")]
impl PostsApi {
    #[oai(method = "get", path = "/:post_id")]
    async fn get_post(&self, post_id: Path<String>, db: Data<&DatabaseConnection>) -> Result<GetPostResponse> {
        let post: Option<entities::posts::Model> = Posts::find_by_id(post_id.0).one(*db).await.map_err(InternalServerError)?;

        match post {
            Some(post) => {
                return Ok(GetPostResponse::Ok(Json(post)))
            },
            None => {
                return Ok(GetPostResponse::NotFound)
            }
        }
    }
}