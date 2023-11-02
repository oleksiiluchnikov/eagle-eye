use super::client::EagleClient;
use serde_json::{json, Value};
use hyper::{Body, Method};
use super::types::*;
use std::error::Error;
use std::path::Path;
use hyper::Uri;


// Application
pub struct ApplicationRequest<'a> {
    client: &'a EagleClient,
}

impl<'a> ApplicationRequest<'a> {
    const RESOURCE: &'static str = "application"; 
    pub fn new(client: &'a EagleClient) -> Self {
        ApplicationRequest { 
            client,
        }
    }
    pub async fn info(&self) -> Result<GetApplicationInfoResult, Box<dyn Error>> {
        let uri = self.client.endpoint(Self::RESOURCE, "info", None)?;
        self.client.execute_request(uri, Method::GET, Body::empty()).await
    }
}


// Folder

pub struct FolderRequest<'a> {
    client: &'a EagleClient,
    data: Option<Value>,
}

impl<'a> FolderRequest<'a> {
    const RESOURCE: &'static str = "folder";

    pub fn new(client: &'a EagleClient) -> Self {
        FolderRequest {
            client,
            data: None,
        }
    }

    pub async fn list(&self) -> Result<GetFolderListResult, Box<dyn Error>> {
        let uri: Uri = self.client.endpoint(Self::RESOURCE, "list", None)?;
        self.client.execute_request(uri, Method::GET, Body::empty()).await
    }

    pub async fn rename(
        &self,
        folder_id: u64,
        new_name: String,
    ) -> Result<RenameFolderResult, Box<dyn Error>> {
        let data = json!({
            "folder_id": folder_id,
            "new_name": new_name,
        });
        let uri = self.client.endpoint(Self::RESOURCE, "rename", None)?;
        self.client.execute_request(uri, Method::POST, Body::from(serde_json::to_string(&data)?)).await
    }
}

// Item

pub struct ItemRequest<'a> {
    client: &'a EagleClient,
}

impl<'a> ItemRequest<'a> {
    const RESOURCE: &'static str = "item";

    pub fn new(client: &'a EagleClient) -> Self {
        ItemRequest { client }
    }

    pub async fn info(&self, query_params: GetItemInfoParams) -> Result<GetItemInfoResult, Box<dyn Error>> {
        let uri: Uri = self.client.endpoint(Self::RESOURCE, "info", Some(query_params.to_query_string()))?;
        self.client.execute_request(uri, Method::GET, Body::empty()).await
    }

    pub async fn list(&self, query_params: GetItemListParams) -> Result<GetItemListResult, Box<dyn Error>> {
        let uri = self.client.endpoint(Self::RESOURCE, "list", Some(query_params.to_query_string()))?;
        self.client.execute_request(uri, Method::GET, Body::empty()).await
    }

    pub async fn thumbnail(&self, query_params: GetItemThumbnailParams) -> Result<GetItemThumbnailResult, Box<dyn Error>> {
        let uri: Uri = self.client.endpoint(Self::RESOURCE, "thumbnail", Some(query_params.to_query_string()))?;
        self.client.execute_request(uri, Method::GET, Body::empty()).await
    }
}

// Library

pub struct LibraryRequest<'a> {
    client: &'a EagleClient,
    data: Option<Value>,
}

impl<'a> LibraryRequest<'a> {
    const RESOURCE: &'static str = "library";

    pub fn new(client: &'a EagleClient) -> Self {
        LibraryRequest {
            client,
            data: None,
        }
    }

    pub async fn info(&self) -> Result<GetLibraryInfoResult, Box<dyn Error>> {
        let uri = self.client.endpoint(Self::RESOURCE, "info", None)?;
        self.client.execute_request(uri, Method::GET, Body::empty()).await
    }

    pub async fn history(&self) -> Result<GetLibraryHistoryResult, Box<dyn Error>> {
        let uri = self.client.endpoint(Self::RESOURCE, "history", None)?;
        self.client.execute_request(uri, Method::GET, Body::empty()).await
    }

    pub async fn switch(
        &self,
        library_path: &Path,
    ) -> Result<SwitchLibraryResult, Box<dyn Error>> {
        let data = json!({
            "library_path": library_path,
        });
        let uri = self.client.endpoint(Self::RESOURCE, "switch", None)?;
        self.client.execute_request(uri, Method::POST, Body::from(serde_json::to_string(&data)?)).await
    }
}
