use std::collections::HashMap;
use std::marker::PhantomData;
use serde::Deserialize;
use reginleif_macro::{Cache};
use reginleif_utils::save_path::BaseStorePoint;

#[derive(Debug,Clone,Deserialize,PartialEq)]
pub struct AssetObject {
    pub hash:String,
    pub size:i64,
}

impl From<AssetObject> for String{
    fn from(value: AssetObject) -> Self {
        format!("{}:{}",value.hash,value.size)
    }
}

#[derive(Debug,Clone,Deserialize,PartialEq,Cache)]
pub struct AssetInfo<T> where T:BaseStorePoint{
    pub objects:HashMap<String,AssetObject>,
    _t:PhantomData<T>
}

#[derive(Debug,Clone,Deserialize,PartialEq)]
pub struct AssetIndex{
    pub id:String,
    pub sha1:String,
    pub size:i64,
    pub url:String
}