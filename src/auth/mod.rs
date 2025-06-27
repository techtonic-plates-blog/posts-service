use poem::{
 Request
};
use poem_openapi::{

 SecurityScheme,
    auth::Bearer
};
use serde::{Deserialize, Serialize};

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Claims {
    pub sub: String,
    pub company: String,
    pub exp: usize,
}


#[derive(SecurityScheme)]
#[oai(
    ty = "bearer",
    key_in = "header",
    key_name = "Bearer",
    checker="key_checker"
)]
#[allow(dead_code)]
pub struct BearerAuthorization(pub Claims);

async fn key_checker(req: &Request, token: Bearer) -> Option<Claims> {


    return Some(Default::default())
}