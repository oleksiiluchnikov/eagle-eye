use super::api::{
    ApplicationRequest,
    FolderRequest,
    ItemRequest,
    LibraryRequest,
};
use hyper::{Body, Client, Request, Uri};
use hyper::http::uri::Authority;
use hyper::client::HttpConnector;
use hyper::StatusCode;
use serde::Deserialize;
use std::error::Error;

// Error

pub struct EagleClient {
    authority: Authority,
    http_client: Client<HttpConnector>,
}

impl EagleClient {

    pub fn new(host: &str, port: u16) -> Self {
        EagleClient { 
            authority: Authority::from_maybe_shared(format!("{}:{}", host, port)).unwrap(),
            http_client: Client::new() }
    }

    pub fn endpoint(&self, resource: &str, action: &str) -> Result<Uri, Box<dyn Error>> {
        Ok(Uri::builder()
            .scheme("http")
            .authority(self.authority.as_str())
            .path_and_query(format!("/api/{}/{}", resource, action).as_str())
            .build()?)
    }

    pub async fn execute_request<T: for<'de> Deserialize<'de>>(
        &self, 
        uri: Uri, 
        method: hyper::Method,
        body: Body,
    ) -> Result<T, Box<dyn Error>> {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(body)?;

        let response = self.http_client.request(request).await?;
        if response.status() != StatusCode::OK {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Server returned an error",
            )));
        }

        decode_body(response).await
    }
    pub fn application(&self) -> ApplicationRequest {
        ApplicationRequest::new(&self)
    }

    pub fn folder(&self) -> FolderRequest {
        FolderRequest::new(&self)
    }

    pub fn item(&self) -> ItemRequest {
        ItemRequest::new(&self)
    }

    pub fn library(&self) -> LibraryRequest {
        LibraryRequest::new(&self)
    }
}


async fn decode_body<T: for<'de> Deserialize<'de>>(_res: hyper::Response<Body>) -> Result<T, Box<dyn Error>> {
    let body = hyper::body::to_bytes(_res.into_body()).await?;
    let body_str = String::from_utf8(body.to_vec())?;

    // Deserialize into the expected type
    match serde_json::from_str(&body_str) {
        Ok(parsed) => {
            Ok(parsed)},
        Err(e) => {
            let column = e.column();
            println!("Failed to parse JSON at column: {}", column);

            // Get 50 characters before and after the error column for context
            let start = if column > 500 { column - 500 } else { 0 };
            let end = std::cmp::min(column, body_str.len());
            let context = &body_str[start..end];
            println!("Context: {}", context);

            Err(Box::new(e))
        },
    }
}
