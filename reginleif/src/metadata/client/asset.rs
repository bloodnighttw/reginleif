use std::collections::HashMap;
use std::marker::PhantomData;
use reqwest::Client;
use serde::Deserialize;
use reginleif_macro::{Cache};
use reginleif_utils::save_path::{BaseStorePoint, Cache};
use reginleif_utils::sha::SHA;

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

impl <T> AssetInfo<T> where T:BaseStorePoint+Clone{

     pub async fn fetch(base_on:&T,client: Client,url:&str,id:&str) -> anyhow::Result<Self>{
         Self::builder()
             .base_on(base_on)
             .url(url)
             .add("assets")
             .add("indexes")
             .add(format!("{}.json",id))
             .build_try(client).await // skip check
     }

}

#[derive(Debug,Clone,Deserialize,PartialEq)]
pub struct AssetIndex{
    pub id:String,
    pub sha1:SHA,
    pub size:i64,
    pub url:String
}

impl AssetIndex {
    pub async fn fetch_assets_info<T:BaseStorePoint+Clone>(&self,base_on:&T,client: Client) -> anyhow::Result<AssetInfo<T>>{
        AssetInfo::fetch(base_on,client,&self.url,&self.id).await
    }
}