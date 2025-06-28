use std::ops::{Deref, DerefMut};

use minio::s3::{creds::StaticProvider, http::BaseUrl, Client as MinioClient, ClientBuilder};

pub const TYPST_FILES_BUCKET: &str = "typst-files";

#[derive(Clone)]
pub struct ObjectStorage(MinioClient);

impl ObjectStorage {
    pub fn new(url: String, access_key: String, secret: String) -> Result<Self, minio::s3::error::Error> {
        let provider = StaticProvider::new(&access_key, &secret, None);

        
        let  client = ClientBuilder::new(url.parse::<BaseUrl>()?).provider(Some(Box::new(provider))).build()?;

        Ok(Self(client))
    }
}

impl Deref for ObjectStorage {
    type Target = MinioClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ObjectStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}