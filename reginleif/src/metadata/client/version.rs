use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use reginleif_macro::Cache;
use reginleif_utils::save_path::BaseStorePoint;
use crate::metadata::client::asset::AssetIndex;
use crate::metadata::client::library::{CommonLibrary, Library};
use crate::metadata::client::package::DependencyPackage;

/// For package version info, like: minecraft "1.8.9", fabric-loader "0.15.1",etc.
/// This struct is used to store the version info of a package, but we don't store
/// the package details, like dependencies, libraries, main class, etc.
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo{
    pub recommended:bool,
    pub release_time:String,
    pub sha256:String,
    #[serde(rename="type")]
    pub rtype:Option<String>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub requires:Vec<DependencyPackage>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    conflicts:Vec<DependencyPackage>,
    pub version:String,
    volatile: Option<bool>
}


/// This struct is used to store the version details of a package, like minecraft, fabric-loader, etc.
/// Compared with VersionInfo, this struct contains more details, like the dependencies, libraries, main class, etc.
#[derive(Debug,Clone,Deserialize,PartialEq,Cache)]
#[serde(rename_all = "camelCase")]
pub struct VersionDetails<T> where T:BaseStorePoint{
    pub format_version: i32,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub conflicts:Vec<DependencyPackage>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub requires:Vec<DependencyPackage>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub libraries:Vec<Library>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub maven_files:Vec<Library>, // for forge and neoforge 
    pub name:String,
    pub uid:String,
    pub release_time:String,
    #[serde(rename="type")]
    pub type_:Option<String>, // neoforged hasn't this field
    pub version:String,
    pub volatile: Option<bool>,
    pub main_class:Option<String>,
    pub main_jar:Option<CommonLibrary>,
    pub minecraft_arguments:Option<String>,
    pub asset_index:Option<AssetIndex>,
    _t:PhantomData<T>
}

