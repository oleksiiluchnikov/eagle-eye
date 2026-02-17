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
#[derive(Debug, Serialize, Default)]
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
        Self::default()
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

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Order Display Tests
    // =========================================================================

    #[test]
    fn order_display_manual() {
        assert_eq!(Order::MANUAL.to_string(), "MANUAL");
    }

    #[test]
    fn order_display_createdate() {
        assert_eq!(Order::CREATEDATE.to_string(), "CREATEDATE");
    }

    #[test]
    fn order_display_createdate_desc() {
        assert_eq!(Order::CREATEDATEDESC.to_string(), "-CREATEDATE");
    }

    #[test]
    fn order_display_btime() {
        assert_eq!(Order::BTIME.to_string(), "BTIME");
    }

    #[test]
    fn order_display_mtime() {
        assert_eq!(Order::MTIME.to_string(), "MTIME");
    }

    #[test]
    fn order_display_filesize() {
        assert_eq!(Order::FILESIZE.to_string(), "FILESIZE");
    }

    #[test]
    fn order_display_filesize_reverse() {
        assert_eq!(Order::FILESIZEREVERSE.to_string(), "-FILESIZE");
    }

    #[test]
    fn order_display_name() {
        assert_eq!(Order::NAME.to_string(), "NAME");
    }

    #[test]
    fn order_display_name_reverse() {
        assert_eq!(Order::NAMEREVERSE.to_string(), "-NAME");
    }

    #[test]
    fn order_display_resolution() {
        assert_eq!(Order::RESOLUTION.to_string(), "RESOLUTION");
    }

    #[test]
    fn order_display_resolution_reverse() {
        assert_eq!(Order::RESOLUTIONREVERSE.to_string(), "-RESOLUTION");
    }

    // =========================================================================
    // QueryParams Tests - GetItemListParams
    // =========================================================================

    #[test]
    fn item_list_params_all_none() {
        let params = GetItemListParams::new();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn item_list_params_limit_only() {
        let params = GetItemListParams {
            limit: Some(50),
            ..Default::default()
        };
        assert_eq!(params.to_query_string(), "limit=50");
    }

    #[test]
    fn item_list_params_offset_only() {
        let params = GetItemListParams {
            offset: Some(100),
            ..Default::default()
        };
        assert_eq!(params.to_query_string(), "offset=100");
    }

    #[test]
    fn item_list_params_order_by_only() {
        let params = GetItemListParams {
            order_by: Some(Order::FILESIZE),
            ..Default::default()
        };
        assert_eq!(params.to_query_string(), "orderBy=FILESIZE");
    }

    #[test]
    fn item_list_params_order_by_reverse() {
        let params = GetItemListParams {
            order_by: Some(Order::FILESIZEREVERSE),
            ..Default::default()
        };
        // The `-` character gets percent-encoded to `%2D` which is correct
        assert_eq!(params.to_query_string(), "orderBy=%2DFILESIZE");
    }

    #[test]
    fn item_list_params_keyword_with_special_chars() {
        let params = GetItemListParams {
            keyword: Some("hello world & more".to_string()),
            ..Default::default()
        };
        assert!(params.to_query_string().contains("keyword="));
        // Special chars should be percent-encoded
        assert!(params.to_query_string().contains("%26")); // & encoded
    }

    #[test]
    fn item_list_params_multiple_fields() {
        let params = GetItemListParams {
            limit: Some(25),
            offset: Some(50),
            order_by: Some(Order::CREATEDATE),
            ..Default::default()
        };
        let query = params.to_query_string();
        assert!(query.contains("limit=25"));
        assert!(query.contains("offset=50"));
        assert!(query.contains("orderBy=CREATEDATE"));
    }

    #[test]
    fn item_list_params_ext_filter() {
        let params = GetItemListParams {
            ext: Some("png".to_string()),
            ..Default::default()
        };
        assert_eq!(params.to_query_string(), "ext=png");
    }

    #[test]
    fn item_list_params_tags_filter() {
        let params = GetItemListParams {
            tags: Some("Design, Poster".to_string()),
            ..Default::default()
        };
        assert_eq!(params.to_query_string(), "tags=Design%2C%20Poster");
    }

    #[test]
    fn item_list_params_folders_filter() {
        let params = GetItemListParams {
            folders: Some("KAY6NTU6UYI5Q,KBJ8Z60O88VMG".to_string()),
            ..Default::default()
        };
        assert_eq!(
            params.to_query_string(),
            "folders=KAY6NTU6UYI5Q%2CKBJ8Z60O88VMG"
        );
    }

    // =========================================================================
    // QueryParams Tests - GetItemInfoParams
    // =========================================================================

    #[test]
    fn item_info_params_basic() {
        let params = GetItemInfoParams {
            id: "ABC123".to_string(),
        };
        assert_eq!(params.to_query_string(), "id=ABC123");
    }

    #[test]
    fn item_info_params_with_special_chars() {
        let params = GetItemInfoParams {
            id: "folder/name".to_string(),
        };
        assert_eq!(params.to_query_string(), "id=folder%2Fname");
    }

    // =========================================================================
    // QueryParams Tests - GetItemThumbnailParams
    // =========================================================================

    #[test]
    fn item_thumbnail_params_basic() {
        let params = GetItemThumbnailParams {
            id: "XYZ789".to_string(),
        };
        assert_eq!(params.to_query_string(), "id=XYZ789");
    }

    // =========================================================================
    // QueryParams Tests - HashMap
    // =========================================================================

    #[test]
    fn hashmap_params_empty() {
        let map: HashMap<&str, &str> = HashMap::new();
        assert_eq!(map.to_query_string(), "");
    }

    #[test]
    fn hashmap_params_single() {
        let mut map = HashMap::new();
        map.insert("key", "value");
        assert_eq!(map.to_query_string(), "key=value");
    }

    #[test]
    fn hashmap_params_multiple() {
        let mut map = HashMap::new();
        map.insert("a", "1");
        map.insert("b", "2");
        // HashMap iteration order is not guaranteed, so check both possibilities
        let result = map.to_query_string();
        assert!(result == "a=1&b=2" || result == "b=2&a=1");
    }

    // =========================================================================
    // Serde Round-trip Tests
    // =========================================================================

    #[test]
    fn status_deserialization_success() {
        let json = r#""success""#;
        let status: Status = serde_json::from_str(json).unwrap();
        assert!(matches!(status, Status::Success));
    }

    #[test]
    fn status_deserialization_error() {
        let json = r#""error""#;
        let status: Status = serde_json::from_str(json).unwrap();
        assert!(matches!(status, Status::Error));
    }

    #[test]
    fn application_info_roundtrip() {
        let json = r#"{
            "status": "success",
            "data": {
                "version": "4.0.0",
                "prereleaseVersion": "beta-1",
                "buildVersion": "20240801",
                "execPath": "/Applications/Eagle.app",
                "platform": "darwin"
            }
        }"#;
        let result: GetApplicationInfoResult = serde_json::from_str(json).unwrap();
        assert!(matches!(result.status, Status::Success));
        assert_eq!(result.data.version, "4.0.0");
        assert_eq!(result.data.build_version, "20240801");
        assert_eq!(result.data.platform, "darwin");
        assert_eq!(
            result.data.exec_path,
            Some("/Applications/Eagle.app".to_string())
        );
    }

    #[test]
    fn folder_list_roundtrip() {
        let json = r#"{
            "status": "success",
            "data": [
                {
                    "id": "FOLDER001",
                    "name": "Design",
                    "images": [],
                    "folders": [],
                    "modificationTime": 1699999999,
                    "editable": true,
                    "tags": ["work", "design"],
                    "children": [],
                    "isExpand": true,
                    "size": 1024,
                    "vstype": "folder",
                    "isVisible": true,
                    "imageCount": 42,
                    "descendantImageCount": 100,
                    "pinyin": "sheji"
                }
            ]
        }"#;
        let result: GetFolderListResult = serde_json::from_str(json).unwrap();
        assert!(matches!(result.status, Status::Success));
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].name, "Design");
        assert_eq!(result.data[0].modification_time, 1699999999);
    }

    #[test]
    fn folder_list_with_nested_children() {
        let json = r#"{
            "status": "success",
            "data": [
                {
                    "id": "PARENT01",
                    "name": "Parent",
                    "modificationTime": 1699999999,
                    "editable": true,
                    "tags": [],
                    "children": [
                        {
                            "id": "CHILD001",
                            "name": "Child",
                            "modificationTime": 1699999888,
                            "editable": false,
                            "tags": ["nested"],
                            "children": []
                        }
                    ]
                }
            ]
        }"#;
        let result: GetFolderListResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.data[0].children.len(), 1);
        assert_eq!(result.data[0].children[0].name, "Child");
    }

    #[test]
    fn item_list_roundtrip() {
        let json = r#"{
            "status": "success",
            "data": [
                {
                    "id": "ITEM001",
                    "name": "screenshot",
                    "size": 2048000,
                    "ext": "png",
                    "tags": ["screenshot", "work"],
                    "folders": ["FOLDER001"],
                    "isDeleted": false,
                    "url": "https://example.com/image.png",
                    "annotation": "Important screenshot",
                    "modificationTime": 1699999999,
                    "height": 1080,
                    "width": 1920,
                    "lastModified": 1699999000,
                    "palettes": [
                        {
                            "color": [255, 0, 0],
                            "ratio": 0.45,
                            "$$hashKey": "palette1"
                        }
                    ]
                }
            ]
        }"#;
        let result: GetItemListResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].name, "screenshot");
        assert_eq!(result.data[0].width, Some(1920));
        assert_eq!(result.data[0].palettes.as_ref().unwrap()[0].ratio, 0.45);
    }

    #[test]
    fn library_info_roundtrip() {
        let json = r#"{
            "status": "success",
            "data": {
                "folders": [],
                "smartFolders": [],
                "quickAccess": [],
                "tagsGroups": [],
                "modificationTime": 1699999999,
                "applicationVersion": "4.0.0",
                "library": {
                    "path": "/Users/test/Library/Eagle",
                    "name": "My Library"
                }
            }
        }"#;
        let result: GetLibraryInfoResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.data.library.name, "My Library");
        assert_eq!(result.data.library.path, "/Users/test/Library/Eagle");
        assert_eq!(result.data.application_version, "4.0.0");
    }

    #[test]
    fn library_history_roundtrip() {
        let json = r#"{
            "status": "success",
            "data": [
                "/Users/test/Library/Eagle",
                "/Users/test/Documents/EagleBackup"
            ]
        }"#;
        let result: GetLibraryHistoryResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0], "/Users/test/Library/Eagle");
    }

    #[test]
    fn item_info_roundtrip() {
        let json = r#"{
            "status": "success",
            "data": {
                "id": "ITEM123",
                "name": "logo-design",
                "size": 1024000,
                "ext": "png",
                "tags": ["branding", "logo"],
                "folders": ["FOLDER001", "FOLDER002"],
                "isDeleted": false,
                "url": "",
                "annotation": "Company logo",
                "modificationTime": 1699999999,
                "width": 512,
                "height": 512,
                "noThumbnail": false,
                "lastModified": 1699999000,
                "palettes": []
            }
        }"#;
        let result: GetItemInfoResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.data.id, "ITEM123");
        assert_eq!(result.data.width, 512);
        assert!(!result.data.is_deleted);
    }

    #[test]
    fn color_serde_roundtrip() {
        // Test that Color enum deserializes from strings
        let red: Color = serde_json::from_str(r#""Red""#).unwrap();
        assert_eq!(red, Color::Red);

        let blue: Color = serde_json::from_str(r#""Blue""#).unwrap();
        assert_eq!(blue, Color::Blue);
    }

    #[test]
    fn create_folder_result_roundtrip() {
        let json = r#"{
            "status": "success",
            "data": {
                "id": "NEWFOLDER01",
                "name": "New Folder",
                "images": [],
                "folders": [],
                "modificationTime": 1699999999,
                "imageMappings": null,
                "tags": [],
                "children": [],
                "isExpand": true
            }
        }"#;
        let result: CreateFolderResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.data.name, "New Folder");
        assert_eq!(result.data.id, "NEWFOLDER01");
    }

    #[test]
    fn switch_library_result_roundtrip() {
        let json = r#"{"status": "success"}"#;
        let result: SwitchLibraryResult = serde_json::from_str(json).unwrap();
        assert!(matches!(result.status, Status::Success));
    }
}
