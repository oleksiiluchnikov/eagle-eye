use super::api::{ApplicationRequest, FolderRequest, ItemRequest, LibraryRequest};
use hyper::client::HttpConnector;
use hyper::http::uri::Authority;
use hyper::StatusCode;
use hyper::{Body, Client, Request, Uri};
use std::error::Error;
use serde::Deserialize;

// Error

/// Client for communicating with the Eagle server
pub struct EagleClient {
    authority: Authority,
    http_client: Client<HttpConnector>,
}

impl EagleClient {
    /// Create a new client
    pub fn new(host: &str, port: u16) -> Self {
        EagleClient {
            authority: Authority::from_maybe_shared(format!("{}:{}", host, port)).unwrap(),
            http_client: Client::new(),
        }
    }

    pub fn endpoint(
        &self,
        resource: &str,
        action: &str,
        query_params: Option<String>,
    ) -> Result<Uri, Box<dyn std::error::Error>> {

    let query_string = query_params.map_or("".to_string(), |params| format!("?{}", params));

    let path_and_query = format!("/api/{}/{}{}", resource, action, query_string);

        Ok(Uri::builder()
            .scheme("http")
            .authority(self.authority.as_str())
            .path_and_query(path_and_query.as_str())
            .build()?)
    }

    /// Execute a request and deserialize the response body
pub async fn execute_request<T: for<'de> Deserialize<'de>>(
    &self,
    uri: Uri,
    method: hyper::Method,
    body: Body,
) -> Result<T, Box<dyn Error>> {
    let request = Request::builder().method(method).uri(uri).body(body)?;

    let response = self.http_client.request(request).await?;
    if response.status() != StatusCode::OK {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Server returned an error",
        )));
    }
    Ok(decode_body(response).await?)
}

    /// Get a request builder for the application resource
    pub fn application(&self) -> ApplicationRequest {
        ApplicationRequest::new(&self)
    }

    /// Get a request builder for the folder resource
    pub fn folder(&self) -> FolderRequest {
        FolderRequest::new(&self)
    }

    /// Get a request builder for the item resource
    pub fn item(&self) -> ItemRequest {
        ItemRequest::new(&self)
    }

    /// Get a request builder for the library resource
    pub fn library(&self) -> LibraryRequest {
        LibraryRequest::new(&self)
    }
}

/// Decode the body of a response into the expected type
async fn decode_body<T: for<'de> Deserialize<'de>>(
    _res: hyper::Response<Body>,
) -> Result<T, Box<dyn Error>> {
    let body = hyper::body::to_bytes(_res.into_body()).await?;
    let body_str = String::from_utf8(body.to_vec())?;

    // Deserialize into the expected type
    match serde_json::from_str(&body_str) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            let column = e.column();
            println!("Failed to parse JSON at column: {}", column);

            // Get 50 characters before and after the error column for context
            let start = if column > 500 { column - 500 } else { 0 };
            let end = std::cmp::min(column, body_str.len());
            let context = &body_str[start..end];
            println!("Context: {}", context);

            Err(Box::new(e))
        }
    }
}
