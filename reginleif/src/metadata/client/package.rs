use std::marker::PhantomData;
use std::path::PathBuf;
use std::slice::Iter;
use serde::{Deserialize, Serialize};
use reginleif_macro::{Cache, Storage};
use reginleif_utils::save_path::{BaseStorePoint, ExpandStorePoint};
use reginleif_utils::sha::SHA;
use crate::metadata::client::version::VersionInfo;

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackageInfo{
    pub name:String,
    pub sha256:SHA,
    pub uid:String,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize,Storage)]
#[filepath(&["packages.json"])]
pub struct PackageList<T> where T:BaseStorePoint{
    pub format_version:i32,
    pub packages:Vec<PackageInfo>,
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

/// This struct is used to store the required or conflict package information.
#[derive(Debug,Clone,Deserialize,Serialize,PartialEq)]
pub struct DependencyPackage {
    pub suggests:Option<String>,
    pub equals:Option<String>,
    pub uid: String
}

/// For package details, like: minecraft, fabric-loader, etc.
/// This struct is used to store the package details, like the name, uid, versions, etc.
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Cache)]
#[serde(rename_all = "camelCase")]
pub struct PackageDetails<T> where T:BaseStorePoint {
    pub format_version:i32,
    pub name:String,
    pub uid:String,
    pub versions:Vec<VersionInfo>,
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
    fn iter(&self) -> Iter<'_, VersionInfo> {
        self.versions.iter()
    }
}