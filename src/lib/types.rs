use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

pub trait QueryParams {
    fn to_query_string(&self) -> String;
}
impl QueryParams for HashMap<&str, &str> {
    fn to_query_string(&self) -> String {
        self.iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>()
            .join("&")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum Color {
    Red,
    Orange,
    Green,
    Yellow,
    Aqua,
    Blue,
    Purple,
    Pink,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetApplicationInfoResult {
    pub status: Status,
    pub data: ApplicationData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationData {
    pub version: String,
    pub prerelease_version: Option<String>,
    #[serde(rename = "buildVersion")]
    pub build_version: String,
    #[serde(rename = "execPath")]
    pub exec_path: Option<String>,
    pub platform: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Child {
    pub id: String,
    pub name: String,
    pub images: Option<Vec<Value>>,
    pub folders: Option<Vec<Value>>,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    pub editable: Option<bool>,
    // pub imagesMappings: Option<Vec<Value>>,
    pub tags: Vec<String>,
    pub children: Vec<Child>,
    #[serde(rename = "isExpand")]
    pub is_expand: Option<bool>,
    pub size: Option<u64>,
    pub vstype: Option<String>,
    pub styles: Option<Styles>,
    #[serde(rename = "isVisible")]
    pub is_visible: Option<bool>,
    pub index: Option<u64>,
    #[serde(rename = "newFolderName")]
    pub new_folder_name: Option<String>,
    #[serde(rename = "imageCount")]
    pub image_count: Option<u64>,
    #[serde(rename = "descendantImageCount")]
    pub descendant_image_count: Option<u64>,
    pub pinyin: Option<String>,
    #[serde(rename = "extendTags")]
    pub extend_tags: Option<Vec<Value>>,
    pub covers: Option<Vec<Value>>,
    pub parent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Styles {
    pub depth: u64,
    pub first: bool,
    pub last: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFolderResult {
    pub status: Status,
    pub data: CreateFolderData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFolderData {
    pub id: String,
    pub name: String,
    pub images: Vec<Value>,
    pub folders: Vec<Value>,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    #[serde(rename = "imageMappings")]
    pub image_mappings: Value,
    pub tags: Vec<String>,
    pub children: Vec<Child>,
    #[serde(rename = "isExpand")]
    pub is_expand: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameFolderResult {
    pub status: Status,
    pub data: RenameFolderData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameFolderData {
    pub id: String,
    pub name: String,
    pub images: Vec<Value>,
    pub folders: Vec<Value>,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    #[serde(rename = "imageMappings")]
    pub image_mappings: Value,
    pub tags: Vec<String>,
    pub children: Vec<Child>,
    #[serde(rename = "isExpand")]
    pub is_expand: bool,
    pub size: u64,
    pub vstype: String,
    pub styles: Styles,
    #[serde(rename = "isVisible")]
    pub is_visible: bool,
    #[serde(rename = "$$hashKey")]
    pub hash_key_: String,
    #[serde(rename = "newFolderName")]
    pub new_folder_name: String,
    pub editable: bool,
    pub pinyin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFolderResult {
    pub status: Status,
    pub data: UpdateFolderData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFolderData {
    pub id: String,
    pub name: String,
    pub images: Vec<Value>,
    pub folders: Vec<Value>,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    #[serde(rename = "imagesMappings")]
    pub images_mappings: Value,
    pub tags: Vec<String>,
    pub children: Vec<Child>,
    #[serde(rename = "isExpand")]
    pub is_expand: bool,
    pub size: u64,
    pub vstype: String,
    pub styles: Styles,
    #[serde(rename = "isVisible")]
    pub is_visible: bool,
    #[serde(rename = "$$hashKey")]
    pub hash_key_: String,
    #[serde(rename = "newFolderName")]
    pub new_folder_name: String,
    pub editable: bool,
    pub pinyin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetFolderListResult {
    pub status: Status,
    pub data: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct FolderListData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub children: Option<Vec<Child>>,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    pub tags: Vec<String>,
    #[serde(rename = "imageCount")]
    pub image_count: u64,
    #[serde(rename = "descendantImageCount")]
    pub descendant_image_count: Option<u64>,
    pub pinyin: String,
    #[serde(rename = "extendTags")]
    pub extend_tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GetRecentFolderListResult {
    pub status: Status,
    pub data: Vec<RecentFolderListData>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RecentFolderListData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub children: Vec<Child>,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    pub tags: Vec<String>,
    pub password: String,
    #[serde(rename = "passwordTips")]
    pub password_tips: String,
    pub images: Vec<Value>,
    #[serde(rename = "isExpand")]
    pub is_expand: bool,
    #[serde(rename = "newFolderName")]
    pub new_folder_name: String,
    #[serde(rename = "imagesMappings")]
    pub images_mappings: Value,
    #[serde(rename = "imageCount")]
    pub image_count: u64,
    #[serde(rename = "descendantImageCount")]
    pub descendant_image_count: Option<u64>,
    pub pinyin: String,
    #[serde(rename = "extendTags")]
    pub extend_tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AddItemFromUrlResult {
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Item {
    pub url: String,
    pub name: Option<String>,
    pub website: Option<String>,
    pub annotation: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "modificationTime")]
    pub modification_time: Option<u64>,
    // OutgoingHttpHeaders is a type alias for OutgoingHttpHeaders
    pub headers: Option<OutgoingHttpHeaders>,
}

#[allow(dead_code)]
pub type OutgoingHttpHeaders = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AddItemFromUrlsResult {
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AddItemFromPathResult {
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AddItemFromPathsResult {
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AddBookmarkResult {
    pub status: Status,
}

#[derive(Debug, Serialize)]
pub struct GetItemInfoParams {
    pub id: String,
}

impl QueryParams for GetItemInfoParams {
    fn to_query_string(&self) -> String {
        let fields: [(&str, &String); 1] = [("id", &self.id)];

        let query_params: Vec<String> = fields
            .iter()
            .map(|&(param_name, param)| {
                format!(
                    "{}={}",
                    param_name,
                    percent_encode(param.as_bytes(), NON_ALPHANUMERIC)
                )
            })
            .collect();

        query_params.join("&")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetItemInfoResult {
    pub status: Status,
    pub data: ItemInfoData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemInfoData {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub ext: String,
    pub tags: Vec<String>,
    pub folders: Option<Vec<String>>,
    #[serde(rename = "isDeleted")]
    pub is_deleted: bool,
    pub url: String,
    pub annotation: String,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    pub width: u64,
    pub height: u64,
    #[serde(rename = "noThumbnail")]
    pub no_thumbnail: Option<bool>,
    #[serde(rename = "lastModified")]
    pub last_modified: u64,
    pub palettes: Vec<Palettes>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Palettes {
    pub color: Vec<u64>,
    // pub ratio: u64, // or f64
    pub ratio: f64,
    #[serde(rename = "$$hashKey")]
    pub hash_key_: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetItemThumbnailParams {
    pub id: String,
}

impl QueryParams for GetItemThumbnailParams {
    fn to_query_string(&self) -> String {
        let fields: [(&str, &String); 1] = [("id", &self.id)];

        let query_params: Vec<String> = fields
            .iter()
            .map(|&(param_name, param)| {
                format!(
                    "{}={}",
                    param_name,
                    percent_encode(param.as_bytes(), NON_ALPHANUMERIC)
                )
            })
            .collect();

        query_params.join("&")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetItemThumbnailResult {
    pub status: Status,
    pub data: String,
}

pub type ItemThumbnailData = String;

#[derive(Debug, Deserialize, Serialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum Order {
    MANUAL,
    CREATEDATE,
    CREATEDATEDESC,
    BTIME,
    MTIME,
    FILESIZE,
    FILESIZEREVERSE,
    NAME,
    NAMEREVERSE,
    RESOLUTION,
    RESOLUTIONREVERSE,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Order::MANUAL => "MANUAL",
            Order::CREATEDATE => "CREATEDATE",
            Order::CREATEDATEDESC => "-CREATEDATE",
            Order::BTIME => "BTIME",
            Order::MTIME => "MTIME",
            Order::FILESIZE => "FILESIZE",
            Order::FILESIZEREVERSE => "-FILESIZE",
            Order::NAME => "NAME",
            Order::NAMEREVERSE => "-NAME",
            Order::RESOLUTION => "RESOLUTION",
            Order::RESOLUTIONREVERSE => "-RESOLUTION",
        };
        write!(f, "{}", value)
    }
}

/// Represents the parameters for the `/api/item/list` request.
#[derive(Debug, Serialize)]
pub struct GetItemListParams {
    /// The number of items to be displayed. The default number is 200.
    pub limit: Option<usize>,
    /// Offset a collection of results from the API. Start with 0.
    pub offset: Option<usize>,
    /// The sorting order. Use "CREATEDATE", "FILESIZE", "NAME", "RESOLUTION", or add a minus sign for descending order: "-FILESIZE".
    pub order_by: Option<Order>,
    /// Filter by the keyword.
    pub keyword: Option<String>,
    /// Filter by the extension type, e.g., "jpg", "png".
    pub ext: Option<String>,
    /// Filter by tags. Use a comma to divide different tags. E.g., "Design, Poster".
    pub tags: Option<String>,
    /// Filter by Folders. Use a comma to divide folder IDs. E.g., "KAY6NTU6UYI5Q,KBJ8Z60O88VMG".
    pub folders: Option<String>,
}

impl GetItemListParams {
    pub fn new() -> Self {
        GetItemListParams {
            limit: None,
            offset: None,
            order_by: None,
            keyword: None,
            ext: None,
            tags: None,
            folders: None,
        }
    }
}

impl QueryParams for GetItemListParams {
    fn to_query_string(&self) -> String {
        let fields: [(&str, Option<String>); 7] = [
            ("limit", self.limit.as_ref().map(|value| value.to_string())),
            (
                "offset",
                self.offset.as_ref().map(|value| value.to_string()),
            ),
            (
                "orderBy",
                self.order_by.as_ref().map(|value| value.to_string()),
            ),
            (
                "keyword",
                self.keyword.as_ref().map(|value| value.to_string()),
            ),
            ("ext", self.ext.as_ref().map(|value| value.to_string())),
            ("tags", self.tags.as_ref().map(|value| value.to_string())),
            (
                "folders",
                self.folders.as_ref().map(|value| value.to_string()),
            ),
        ];

        let query_params: Vec<String> = fields
            .iter()
            .filter_map(|(param_name, param)| {
                param.as_ref().map(|value| {
                    format!(
                        "{}={}",
                        param_name,
                        percent_encode(value.as_bytes(), NON_ALPHANUMERIC)
                    )
                })
            })
            .collect();

        query_params.join("&") // e.g., "limit=10&offset=0"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetItemListResult {
    pub status: Status,
    pub data: Vec<ItemListData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemListData {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub ext: String,
    pub tags: Vec<String>,
    pub folders: Option<Vec<String>>,
    #[serde(rename = "isDeleted")]
    pub is_deleted: bool,
    pub url: String,
    pub annotation: String,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    pub height: Option<u64>,
    pub width: Option<u64>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<u64>,
    pub palettes: Option<Vec<Palettes>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct MoveItemToTrashResult {
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RefreshItemPaletteResult {
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RefreshThumbnailResult {
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct UpdateItemResult {
    pub status: Status,
    pub data: ItemInfoData,
}

/// Get Library Info
#[derive(Debug, Serialize, Deserialize)]
pub struct GetLibraryInfoResult {
    pub status: Status,
    pub data: LibraryInfoData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryInfoData {
    pub folders: Vec<Folder>,
    #[serde(rename = "smartFolders")]
    pub smart_folders: Vec<SmartFolders>,
    #[serde(rename = "quickAccess")]
    pub quick_access: Vec<Value>,
    #[serde(rename = "tagsGroups")]
    pub tags_groups: Vec<Value>,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    #[serde(rename = "applicationVersion")]
    pub application_version: String,
    pub library: LibraryData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryData {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub description: String,
    pub children: Vec<Child>,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    pub tags: Vec<String>,
    #[serde(rename = "iconColor")]
    pub icon_color: Option<String>,
    pub password: String,
    #[serde(rename = "passwordTips")]
    pub password_tips: String,
    #[serde(rename = "coverId")]
    pub cover_id: Option<String>,
    #[serde(rename = "orderBy")]
    pub order_by: Option<Order>,
    #[serde(rename = "sortIncrease")]
    pub sort_increase: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmartFolders {
    pub id: String,
    pub icon: Option<String>,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "modificationTime")]
    pub modification_time: u64,
    pub conditions: Vec<Conditions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Conditions {
    #[serde(rename = "match")]
    pub match_: String,
    pub rules: Vec<Rules>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rules {
    pub method: String,
    pub property: String,
    pub value: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct QuickAccess {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TagsGroups {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub color: Color,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLibraryHistoryResult {
    pub status: Status,
    pub data: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchLibraryResult {
    pub status: Status,
}
