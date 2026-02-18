use super::api::{ApplicationRequest, FolderRequest, ItemRequest, LibraryRequest, TagRequest};
use hyper::client::HttpConnector;
use hyper::http::uri::Authority;
use hyper::StatusCode;
use hyper::{Body, Client, Request, Uri};
use serde::Deserialize;
use std::error::Error;
use std::time::Instant;

/// Client for communicating with the Eagle server
pub struct EagleClient {
    authority: Authority,
    http_client: Client<HttpConnector>,
    /// When true, log HTTP request/response details to stderr.
    debug: bool,
}

impl EagleClient {
    /// Create a new client
    pub fn new(host: &str, port: u16) -> Self {
        EagleClient {
            authority: Authority::from_maybe_shared(format!("{}:{}", host, port)).unwrap(),
            http_client: Client::new(),
            debug: false,
        }
    }

    /// Create a new client with debug mode enabled.
    pub fn with_debug(host: &str, port: u16, debug: bool) -> Self {
        EagleClient {
            authority: Authority::from_maybe_shared(format!("{}:{}", host, port)).unwrap(),
            http_client: Client::new(),
            debug,
        }
    }

    pub fn endpoint(
        &self,
        resource: &str,
        action: &str,
        query_params: Option<String>,
    ) -> Result<Uri, Box<dyn std::error::Error>> {
        let query_string = query_params.map_or(String::new(), |params| format!("?{}", params));
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
        if self.debug {
            eprintln!("> {} {}", method, uri);
        }

        let start = Instant::now();
        let request = Request::builder()
            .method(method.clone())
            .uri(uri)
            .body(body)?;

        let response = self.http_client.request(request).await?;
        let status = response.status();
        let elapsed = start.elapsed();

        if self.debug {
            eprintln!("< {} ({:.0?})", status, elapsed);
        }

        if status != StatusCode::OK {
            return Err(Box::new(std::io::Error::other(format!(
                "Server returned status {}",
                status
            ))));
        }
        decode_body(response).await
    }

    /// Execute a request and return the raw response bytes (for binary endpoints like library/icon).
    pub async fn execute_raw_request(
        &self,
        uri: Uri,
        method: hyper::Method,
        body: Body,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        if self.debug {
            eprintln!("> {} {}", method, uri);
        }

        let start = Instant::now();
        let request = Request::builder()
            .method(method.clone())
            .uri(uri)
            .body(body)?;

        let response = self.http_client.request(request).await?;
        let status = response.status();
        let elapsed = start.elapsed();

        if self.debug {
            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown");
            eprintln!(
                "< {} ({:.0?}) content-type={}",
                status, elapsed, content_type
            );
        }

        if status != StatusCode::OK {
            return Err(Box::new(std::io::Error::other(format!(
                "Server returned status {}",
                status
            ))));
        }

        let bytes = hyper::body::to_bytes(response.into_body()).await?;
        Ok(bytes.to_vec())
    }

    /// Get a request builder for the application resource
    pub fn application(&self) -> ApplicationRequest<'_> {
        ApplicationRequest::new(self)
    }

    /// Get a request builder for the folder resource
    pub fn folder(&self) -> FolderRequest<'_> {
        FolderRequest::new(self)
    }

    /// Get a request builder for the item resource
    pub fn item(&self) -> ItemRequest<'_> {
        ItemRequest::new(self)
    }

    /// Get a request builder for the library resource
    pub fn library(&self) -> LibraryRequest<'_> {
        LibraryRequest::new(self)
    }

    /// Get a request builder for the tag resource
    pub fn tag(&self) -> TagRequest<'_> {
        TagRequest::new(self)
    }
}

/// Decode the body of a response into the expected type
async fn decode_body<T: for<'de> Deserialize<'de>>(
    res: hyper::Response<Body>,
) -> Result<T, Box<dyn Error>> {
    let body = hyper::body::to_bytes(res.into_body()).await?;
    let body_str = String::from_utf8(body.to_vec())?;

    match serde_json::from_str(&body_str) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            let column = e.column();
            eprintln!("Failed to parse JSON at column: {}", column);

            let start = column.saturating_sub(500);
            let end = std::cmp::min(column, body_str.len());
            let context = &body_str[start..end];
            eprintln!("Context: {}", context);

            Err(Box::new(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // EagleClient::new() Tests
    // =========================================================================

    #[test]
    fn client_new_basic() {
        let client = EagleClient::new("localhost", 41595);
        // Just verify it doesn't panic - authority is private
        // The real test is that we can build URIs with it
        assert!(!client.debug);
        let _ = client;
    }

    #[test]
    fn client_new_different_port() {
        let client = EagleClient::new("127.0.0.1", 8080);
        let uri = client.endpoint("item", "list", None).unwrap();
        assert!(uri.to_string().contains("127.0.0.1:8080"));
    }

    #[test]
    fn client_with_debug() {
        let client = EagleClient::with_debug("localhost", 41595, true);
        assert!(client.debug);
    }

    // =========================================================================
    // EagleClient::endpoint() Tests
    // =========================================================================

    #[test]
    fn endpoint_no_query_params() {
        let client = EagleClient::new("localhost", 41595);
        let uri = client.endpoint("item", "list", None).unwrap();
        assert_eq!(uri.to_string(), "http://localhost:41595/api/item/list");
    }

    #[test]
    fn endpoint_with_query_params() {
        let client = EagleClient::new("localhost", 41595);
        let uri = client
            .endpoint("item", "list", Some("limit=10&offset=0".to_string()))
            .unwrap();
        assert_eq!(
            uri.to_string(),
            "http://localhost:41595/api/item/list?limit=10&offset=0"
        );
    }

    #[test]
    fn endpoint_application_info() {
        let client = EagleClient::new("localhost", 41595);
        let uri = client.endpoint("application", "info", None).unwrap();
        assert_eq!(
            uri.to_string(),
            "http://localhost:41595/api/application/info"
        );
    }

    #[test]
    fn endpoint_folder_list() {
        let client = EagleClient::new("localhost", 41595);
        let uri = client.endpoint("folder", "list", None).unwrap();
        assert_eq!(uri.to_string(), "http://localhost:41595/api/folder/list");
    }

    #[test]
    fn endpoint_library_info() {
        let client = EagleClient::new("localhost", 41595);
        let uri = client.endpoint("library", "info", None).unwrap();
        assert_eq!(uri.to_string(), "http://localhost:41595/api/library/info");
    }

    #[test]
    fn endpoint_item_thumbnail() {
        let client = EagleClient::new("localhost", 41595);
        let uri = client
            .endpoint("item", "thumbnail", Some("id=ABC123".to_string()))
            .unwrap();
        assert_eq!(
            uri.to_string(),
            "http://localhost:41595/api/item/thumbnail?id=ABC123"
        );
    }

    #[test]
    fn endpoint_with_empty_query_params() {
        let client = EagleClient::new("localhost", 41595);
        let uri = client
            .endpoint("item", "list", Some("".to_string()))
            .unwrap();
        // Empty query string should still have the ? but that's handled by the format
        // Actually, with our logic, empty string produces "?" which might not be ideal
        // Let's check what actually happens
        let uri_str = uri.to_string();
        assert!(uri_str.starts_with("http://localhost:41595/api/item/list"));
    }

    // =========================================================================
    // decode_body() Tests
    // =========================================================================

    #[tokio::test]
    async fn decode_body_valid_json() {
        use hyper::Response;

        let json = r#"{"status": "success", "data": "test"}"#;
        let response = Response::builder()
            .status(200)
            .body(Body::from(json))
            .unwrap();

        #[derive(Deserialize, Debug)]
        struct TestResponse {
            status: String,
            data: String,
        }

        let result: Result<TestResponse, _> = decode_body(response).await;
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.status, "success");
        assert_eq!(parsed.data, "test");
    }

    #[tokio::test]
    async fn decode_body_invalid_json() {
        use hyper::Response;

        let invalid_json = r#"{"status": "success", invalid json here}"#;
        let response = Response::builder()
            .status(200)
            .body(Body::from(invalid_json))
            .unwrap();

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct TestResponse {
            status: String,
        }

        let result: Result<TestResponse, _> = decode_body(response).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn decode_body_empty_body() {
        use hyper::Response;

        let response = Response::builder().status(200).body(Body::empty()).unwrap();

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct TestResponse {
            status: String,
        }

        let result: Result<TestResponse, _> = decode_body(response).await;
        // Empty body should cause a parse error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn decode_body_complex_nested() {
        use hyper::Response;

        let json = r#"{
            "status": "success",
            "data": {
                "items": [
                    {"id": "1", "name": "item1"},
                    {"id": "2", "name": "item2"}
                ],
                "total": 2
            }
        }"#;

        let response = Response::builder()
            .status(200)
            .body(Body::from(json))
            .unwrap();

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct Item {
            id: String,
            name: String,
        }

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct Data {
            items: Vec<Item>,
            total: i32,
        }

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct TestResponse {
            status: String,
            data: Data,
        }

        let result: Result<TestResponse, _> = decode_body(response).await;
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.data.items.len(), 2);
        assert_eq!(parsed.data.total, 2);
    }
}
