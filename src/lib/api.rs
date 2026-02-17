use super::client::EagleClient;
use super::types::*;
use hyper::Uri;
use hyper::{Body, Method};
use serde_json::json;
use std::error::Error;
use std::path::Path;

// Application

pub struct ApplicationRequest<'a> {
    client: &'a EagleClient,
}

impl<'a> ApplicationRequest<'a> {
    const RESOURCE: &'static str = "application";

    pub fn new(client: &'a EagleClient) -> Self {
        ApplicationRequest { client }
    }

    pub async fn info(&self) -> Result<GetApplicationInfoResult, Box<dyn Error>> {
        let uri = self.client.endpoint(Self::RESOURCE, "info", None)?;
        self.client
            .execute_request(uri, Method::GET, Body::empty())
            .await
    }
}

// Folder

pub struct FolderRequest<'a> {
    client: &'a EagleClient,
}

impl<'a> FolderRequest<'a> {
    const RESOURCE: &'static str = "folder";

    pub fn new(client: &'a EagleClient) -> Self {
        FolderRequest { client }
    }

    pub async fn list(&self) -> Result<GetFolderListResult, Box<dyn Error>> {
        let uri: Uri = self.client.endpoint(Self::RESOURCE, "list", None)?;
        self.client
            .execute_request(uri, Method::GET, Body::empty())
            .await
    }

    /// Create a new folder.
    ///
    /// - `folder_name`: Name of the folder to create.
    /// - `parent`: Optional parent folder ID.
    pub async fn create(
        &self,
        folder_name: &str,
        parent: Option<&str>,
    ) -> Result<CreateFolderResult, Box<dyn Error>> {
        let mut data = json!({
            "folderName": folder_name,
        });
        if let Some(parent_id) = parent {
            data["parent"] = json!(parent_id);
        }
        let uri = self.client.endpoint(Self::RESOURCE, "create", None)?;
        self.client
            .execute_request(uri, Method::POST, Body::from(serde_json::to_string(&data)?))
            .await
    }

    /// Rename a folder.
    ///
    /// - `folder_id`: ID of the folder to rename.
    /// - `new_name`: The new name for the folder.
    pub async fn rename(
        &self,
        folder_id: &str,
        new_name: &str,
    ) -> Result<RenameFolderResult, Box<dyn Error>> {
        let data = json!({
            "folderId": folder_id,
            "newName": new_name,
        });
        let uri = self.client.endpoint(Self::RESOURCE, "rename", None)?;
        self.client
            .execute_request(uri, Method::POST, Body::from(serde_json::to_string(&data)?))
            .await
    }

    /// Update a folder's properties.
    ///
    /// - `folder_id`: ID of the folder to update.
    /// - `new_name`: Optional new name.
    /// - `new_description`: Optional new description.
    /// - `new_color`: Optional new color.
    pub async fn update(
        &self,
        folder_id: &str,
        new_name: Option<&str>,
        new_description: Option<&str>,
        new_color: Option<&str>,
    ) -> Result<UpdateFolderResult, Box<dyn Error>> {
        let mut data = json!({
            "folderId": folder_id,
        });
        if let Some(name) = new_name {
            data["newName"] = json!(name);
        }
        if let Some(desc) = new_description {
            data["newDescription"] = json!(desc);
        }
        if let Some(color) = new_color {
            data["newColor"] = json!(color);
        }
        let uri = self.client.endpoint(Self::RESOURCE, "update", None)?;
        self.client
            .execute_request(uri, Method::POST, Body::from(serde_json::to_string(&data)?))
            .await
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

    pub async fn info(
        &self,
        query_params: GetItemInfoParams,
    ) -> Result<GetItemInfoResult, Box<dyn Error>> {
        let uri: Uri =
            self.client
                .endpoint(Self::RESOURCE, "info", Some(query_params.to_query_string()))?;
        self.client
            .execute_request(uri, Method::GET, Body::empty())
            .await
    }

    pub async fn list(
        &self,
        query_params: GetItemListParams,
    ) -> Result<GetItemListResult, Box<dyn Error>> {
        let uri =
            self.client
                .endpoint(Self::RESOURCE, "list", Some(query_params.to_query_string()))?;
        self.client
            .execute_request(uri, Method::GET, Body::empty())
            .await
    }

    pub async fn thumbnail(
        &self,
        query_params: GetItemThumbnailParams,
    ) -> Result<GetItemThumbnailResult, Box<dyn Error>> {
        let uri: Uri = self.client.endpoint(
            Self::RESOURCE,
            "thumbnail",
            Some(query_params.to_query_string()),
        )?;
        self.client
            .execute_request(uri, Method::GET, Body::empty())
            .await
    }
}

// Library

pub struct LibraryRequest<'a> {
    client: &'a EagleClient,
}

impl<'a> LibraryRequest<'a> {
    const RESOURCE: &'static str = "library";

    pub fn new(client: &'a EagleClient) -> Self {
        LibraryRequest { client }
    }

    pub async fn info(&self) -> Result<GetLibraryInfoResult, Box<dyn Error>> {
        let uri = self.client.endpoint(Self::RESOURCE, "info", None)?;
        self.client
            .execute_request(uri, Method::GET, Body::empty())
            .await
    }

    pub async fn history(&self) -> Result<GetLibraryHistoryResult, Box<dyn Error>> {
        let uri = self.client.endpoint(Self::RESOURCE, "history", None)?;
        self.client
            .execute_request(uri, Method::GET, Body::empty())
            .await
    }

    /// Switch to a different library.
    ///
    /// - `library_path`: Path to the library to switch to.
    pub async fn switch(&self, library_path: &Path) -> Result<SwitchLibraryResult, Box<dyn Error>> {
        let data = json!({
            "libraryPath": library_path.to_string_lossy(),
        });
        let uri = self.client.endpoint(Self::RESOURCE, "switch", None)?;
        self.client
            .execute_request(uri, Method::POST, Body::from(serde_json::to_string(&data)?))
            .await
    }
}
