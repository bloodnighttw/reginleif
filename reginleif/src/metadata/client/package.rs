use std::marker::PhantomData;
use std::path::PathBuf;
use std::slice::Iter;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use reginleif_macro::{Cache, Storage};
use reginleif_utils::save_path::{BaseStorePoint, Cache, ExpandStorePoint, Store};
use reginleif_utils::sha::SHA;
use crate::metadata::client::version::VersionInfo;

/// This struct represents simple package information.
/// It's used to store the package name, uid, and sha256,
/// which is used to fetch and verify the package details from [PackageDetails].
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackageInfo{
    /// The name of the package, like: minecraft, fabric-loader, etc.
    pub name:String,
    /// The sha256 of the package.
    pub sha256:SHA,
    /// The uid of the package, like: net.minecraft, net.fabricmc.loader, etc.
    pub uid:String,
}

impl PackageInfo{
    pub async fn get_details<T:BaseStorePoint+Clone>(&self,base_on:&T,client: Client,url:&str) -> anyhow::Result<PackageDetails<T>>{
        PackageDetails::fetch(base_on,client,url,self).await
    }
}

/// This struct is used to store the package list.
/// It contains the format version and the list of package info.
///
/// For details, see [PackageInfo].
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize,Storage,Cache)]
#[filepath(&["packages.json"])]
#[serde(rename_all = "camelCase")]
pub struct PackageList<T> where T:BaseStorePoint{
    /// The format version of the package list.
    pub format_version:i32,
    /// The packages list.
    pub packages:Vec<PackageInfo>,
    #[serde(skip)]
    pub _t:PhantomData<T>
}

impl <T> IntoIterator for PackageList<T> where T:BaseStorePoint{
    type Item = PackageInfo;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.packages.into_iter()
    }
}

impl <T> PackageList<T> where T:BaseStorePoint{
    pub fn iter(&self) -> Iter<'_, PackageInfo> {
        self.packages.iter()
    }
}

impl <T> PackageList<T> where T:BaseStorePoint+Clone{
    pub async fn fetch(base_on:&T, client: Client, url: &str) -> anyhow::Result<Self>{
        let mut builder = Self::builder()
            .base_on(base_on)
            .url(url);

        for i in Self::FILE_PATH.iter(){
            builder = builder.add(i);
        }

        builder.build_try(client).await
    }
}

/// This struct is used to store the dependency package.
/// Dependency package has three fields: suggests, equals, and uid.
/// suggest and equals represent the relationship from uid.
///
/// For details, see field description.
#[derive(Debug,Clone,Deserialize,Serialize,PartialEq)]
pub struct DependencyPackage {
    /// when suggest is Some(String), the value of String is the suggested package version.
    pub suggests:Option<String>,
    /// when equals is Some(String), the value of String is the required package version.
    pub equals:Option<String>,
    /// The uid of the package, when both equals and suggests are None, the package is required, but the version is not specified.
    pub uid: String
}

/// This struct represents the package details.
/// It contains the format version, name, uid, and versions of the package.
/// The versions are stored in a vector of [`VersionInfo`].
/// If you want to get the version details, you can use the [`VersionInfo::fetch_detail`] to fetch it.
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Cache)]
#[serde(rename_all = "camelCase")]
pub struct PackageDetails<T> where T:BaseStorePoint {
    /// The format version of the package details.
    pub format_version:i32,
    /// The name of the package, like: minecraft, fabric-loader, etc.
    pub name:String,
    /// the uid of the package, like: net.minecraft, net.fabricmc.loader, etc.
    pub uid:String,
    /// The versions of the package.
    pub versions:Vec<VersionInfo>,
    #[serde(skip)]
    _t:PhantomData<T>
}

impl <T> ExpandStorePoint for PackageDetails<T> where T:BaseStorePoint{
    fn get_suffix(&self) -> PathBuf {
        PathBuf::from(&self.uid).join("index.json")
    }
}

impl <T> IntoIterator for PackageDetails<T> where T:BaseStorePoint{
    type Item = VersionInfo;
    type IntoIter = std::vec::IntoIter<VersionInfo>;

    fn into_iter(self) -> Self::IntoIter {
        self.versions.into_iter()
    }
}

impl <T> PackageDetails<T> where T:BaseStorePoint{
    pub fn iter(&self) -> Iter<'_, VersionInfo> {
        self.versions.iter()
    }
}

impl <T> PackageDetails<T> where T:BaseStorePoint+Clone{
    pub async fn fetch(base_on:&T, client: Client, base_url:&str, package_info: &PackageInfo) -> anyhow::Result<Self>{
        Self::builder()
            .base_on(base_on)
            .url(format!("{}/{}/index.json",base_url,package_info.uid))
            .add(&package_info.uid)
            .add("index.json")
            .build_check(client.clone(),package_info.sha256.clone()).await
    }
}