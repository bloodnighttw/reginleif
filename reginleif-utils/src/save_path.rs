//! The module for the save and load the data from the file.
//! It provides the trait to save and load the data from the file,
//! from the path that is constant to dynamic.

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use reqwest::Client;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use sha1::Digest as Digest1;
use log::log;
use crate::sha::SHA;

/// A trait for the base path of the data.
///
/// You have to implement this trait to provide the base path of the data.
/// You also have to care about the thread safety of the data.
/// The data should be thread safe (Sync+Send).
///
/// You can use the derive macro to implement this trait.
/// You can also implement [From](From) or [TryFrom](TryFrom)
/// to data struct convert more easily.
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use reginleif_macro::BaseStorePoint;
///
/// #[derive(BaseStorePoint)] // the macro only accept one field struct.
/// struct TestPath(PathBuf);
///
/// ```
pub trait BaseStorePoint:Sync+Send{
    /// Get the path of the data.
    fn get_base(&self) -> PathBuf;
}

/// The trait that return the relative path of the data from base.
///
/// This trait is using by [Save](Save) trait to get the relative path of the data
/// from base, and you have to implement this trait to provide the relative path of the data
/// if you are about to use [Save](Save) trait.
/// It is used by the struct which have a path that is not constant, varies from its field, and
/// if you have a constant path, you should use [Store](Store) trait.
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use serde::{Deserialize, Serialize};
/// use reginleif_macro::{BaseStorePoint, Storage};
/// use reginleif_utils::save_path::{Store, Save, Load};
///
/// #[derive(BaseStorePoint,PartialEq,Debug)]
/// struct TestPath(PathBuf);
///
/// impl From<PathBuf> for TestPath{
///    fn from(path:PathBuf) -> Self{
///       Self(path)
///   }
/// }
/// ```
pub trait ExpandStorePoint{
    fn get_suffix(&self) -> PathBuf;
}

/// A trait for the store of the data which have a const path.
///
/// You have to implement this trait to provide the store path of the data.
/// When compared to [Save](Save) and [Load](Load) trait, this trait is used for the data which have a constant path.
///
/// You can also use the derive macro to implement this trait.
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use serde::{Deserialize, Serialize};
/// use reginleif_macro::{BaseStorePoint, Storage};
/// use reginleif_utils::save_path::{Store, Save, Load};
///
/// #[derive(BaseStorePoint,PartialEq,Debug)]
/// struct TestPath(PathBuf);
///
/// impl From<PathBuf> for TestPath{
///    fn from(path:PathBuf) -> Self{
///       Self(path)
///   }
/// }
///
/// #[derive(Deserialize,Serialize,PartialEq,Debug,Storage)]
/// #[base_on(TestPath)] #[filepath(&["test.txt"])] // the file will store in TestPath + test.txt
/// struct A;
///
///
/// ```
///
/// The macro also support AcceptStorePoint as a generic type,
/// so you can use the generic type to store the data with different base path.
///
/// # Example
/// ```no_run
/// use std::marker::PhantomData;
/// use std::path::PathBuf;
/// use serde::{Deserialize, Serialize};
/// use reginleif_macro::{BaseStorePoint, Storage};
/// use reginleif_utils::save_path::{Store, Save, Load, BaseStorePoint};
///
/// #[derive(BaseStorePoint,PartialEq,Debug)]
/// struct TestPath(PathBuf);
///
/// impl From<PathBuf> for TestPath{
///     fn from(path:PathBuf) -> Self{
///         Self(path)
///     }
/// }
///
/// #[derive(Deserialize,Serialize,PartialEq,Debug,Storage)]
/// #[filepath(&["test.txt"])]
/// struct A<T> where T:BaseStorePoint{
///     num:String,
///     _t:PhantomData<T>
/// }
///
/// ```
///
/// Now you can store the data with different base path with generic type.
pub trait Store:Serialize+DeserializeOwned{

    /// The const path of the file.
    /// Separate the path by the array of the str.
    const FILE_PATH:&'static [&'static str];
    /// The type of the base path you have to accept.
    /// This field will become save and load function's argument.
    type AcceptStorePoint:BaseStorePoint;

    /// Get the full path of the data.
    fn full_path(base:&Self::AcceptStorePoint) -> PathBuf{
        let mut base_path = base.get_base();
        for i in Self::FILE_PATH{
            base_path = base_path.join(i);
        }
        base_path
    }

    /// Save the data to the file.
    ///
    /// # Arguments
    /// * `base`: the base path of the data.
    fn save(&self, base: &Self::AcceptStorePoint) -> anyhow::Result<()> {
        let base_path = Self::full_path(&base);

        std::fs::create_dir_all(base_path.parent().ok_or(anyhow::anyhow!("No parent"))?)?;
        std::fs::write(base_path,serde_json::to_string(self)?.as_bytes())?;

        Ok(())

    }

    /// Load the data from the file.
    ///
    /// # Arguments
    /// * `base`: the base path of the data.
    fn load(base: &Self::AcceptStorePoint) -> anyhow::Result<Self> {

        let base_path = Self::full_path(&base);

        let json = std::fs::read_to_string(base_path)?;
        Ok(serde_json::from_str(&json)?)
    }
}

/// A trait for the save the data which have a dynamic path.
///
/// You have to implement this trait to save the data to the file.
/// When compared to [Store](Store) trait, this trait is used for the data which have a dynamic path.
///
/// You can also use the derive macro to implement this trait.
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use serde::{Deserialize, Serialize};
/// use reginleif_macro::{BaseStorePoint, Save, Load};
/// use reginleif_utils::save_path::{ExpandStorePoint, Save, Load};
///
/// #[derive(BaseStorePoint,PartialEq,Debug)]
/// struct TestPath(PathBuf);
///
/// impl From<PathBuf> for TestPath{
///     fn from(path:PathBuf) -> Self{
///         Self(path)
///     }
/// }
///
/// #[derive(Serialize,Deserialize,Save,Load,PartialEq,Debug)]
/// #[base_on(TestPath)]
/// struct B(i32);
///
/// impl ExpandStorePoint for B{ // you should implement this trait to provide the relative path of the data from base.
///     fn get_suffix(&self) -> PathBuf {
///         PathBuf::from(format!("{}.json",self.0))
///     }
/// }
/// ```
///
/// The macro also support AcceptStorePoint as a generic type,
/// so you can use the generic type to save the data with different base path.
/// Note the generic argument should be the struct that impl [BaseStorePoint](BaseStorePoint) trait
/// and only one.
///
/// # Example
/// ```no_run
/// use std::marker::PhantomData;
/// use std::path::PathBuf;
/// use serde::{Deserialize, Serialize};
/// use reginleif_macro::{BaseStorePoint, Load, Save, Storage};
/// use reginleif_utils::save_path::{Store, Save, Load, BaseStorePoint, ExpandStorePoint};
///
/// #[derive(BaseStorePoint,PartialEq,Debug)]
/// struct TestPath(PathBuf);
///
/// impl From<PathBuf> for TestPath{
///     fn from(path:PathBuf) -> Self{
///         Self(path)
///     }
/// }
///
/// #[derive(Serialize,Deserialize,PartialEq,Debug,Save,Load)]
/// struct C<T> where T:BaseStorePoint{
///     num:String,
///     _t:PhantomData<T>
/// }
///
/// impl <T> ExpandStorePoint for C<T> where T:BaseStorePoint{ // you still need to implement this trait to provide the relative path of the data from base.
///     fn get_suffix(&self) -> PathBuf {
///         PathBuf::from(format!("{}.json",self.num))
///     }
/// }
///
/// ```
pub trait Save:ExpandStorePoint+Serialize{

    /// The type of the base path you have to accept.
    /// This field will become save function's argument.
    type AcceptStorePoint:BaseStorePoint;

    /// Save the data to the file.
    ///
    /// # Arguments
    /// * `base`: the base path of the data.
    fn save(&self, base:&Self::AcceptStorePoint) -> anyhow::Result<()>{
        let base_path = base.get_base().join(&self.get_suffix());

        std::fs::create_dir_all(base_path.parent().ok_or(anyhow::anyhow!("No parent"))?)?;
        std::fs::write(base_path,serde_json::to_string(self)?.as_bytes())?;

        Ok(())
    }
}


/// A trait to load the data which have a dynamic path.
///
/// You have to implement this trait to load the data from the file.
///
/// You can also use the derive macro to implement this trait.
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use serde::{Deserialize, Serialize};
/// use reginleif_macro::{BaseStorePoint, Save, Load};
/// use reginleif_utils::save_path::{ExpandStorePoint, Save, Load};
///
/// #[derive(BaseStorePoint,PartialEq,Debug)]
/// struct TestPath(PathBuf);
///
/// impl From<PathBuf> for TestPath{
///     fn from(path:PathBuf) -> Self{
///         Self(path)
///     }
/// }
///
/// #[derive(Serialize,Deserialize,Save,Load,PartialEq,Debug)]
/// #[base_on(TestPath)]
/// struct B;
///
/// impl ExpandStorePoint for B{ // you should implement this trait to provide the relative path of the data from base.
///     fn get_suffix(&self) -> PathBuf {
///         PathBuf::from("test.txt")
///     }
/// }
/// ```
///
/// The macro also support AcceptStorePoint as a generic type,
/// so you can use the generic type to save the data with different base path.
/// Note the generic argument should be the struct that impl [BaseStorePoint](BaseStorePoint) trait
/// and only one.
///
/// # Example
/// ```no_run
/// use std::marker::PhantomData;
/// use std::path::PathBuf;
/// use serde::{Deserialize, Serialize};
/// use reginleif_macro::{BaseStorePoint, Load, Save, Storage};
/// use reginleif_utils::save_path::{Store, Save, Load, BaseStorePoint, ExpandStorePoint};
///
/// #[derive(BaseStorePoint,PartialEq,Debug)]
/// struct TestPath(PathBuf);
///
/// impl From<PathBuf> for TestPath{
///     fn from(path:PathBuf) -> Self{
///         Self(path)
///     }
/// }
///
/// #[derive(Serialize,Deserialize,PartialEq,Debug,Save,Load)]
/// struct C<T> where T:BaseStorePoint{
///     num:String,
///     _t:PhantomData<T>
/// }
///
/// impl <T> ExpandStorePoint for C<T> where T:BaseStorePoint{ // you still need to implement this trait to provide the relative path of the data from base.
///     fn get_suffix(&self) -> PathBuf {
///         PathBuf::from(format!("{}.json",self.num))
///     }
/// }
///
/// ```

pub trait Load:DeserializeOwned{

    /// The type of the base path you have to accept.
    type AcceptStorePoint:BaseStorePoint;

    /// Load the data from the file.
    ///
    /// # Arguments
    /// * `base`: the base path of the data.
    /// * `suffix`: the relative path of the data from base.
    fn load<P: AsRef<Path>>(base: &Self::AcceptStorePoint, suffix: P) -> anyhow::Result<Self>{
        let path = base.get_base().join(suffix);
        let content = std::fs::read_to_string(path)?;
        // Remove the explicit lifetime annotation from the call to `serde_json::from_str`
        let json = serde_json::from_str(&content)?;
        Ok(json)
    }
}

/// private function to handle the file which is not exist.
async fn handle_file_not_exist(path:&PathBuf, client: &Client, url:&str) -> anyhow::Result<()>{
    tokio::fs::create_dir_all(path.parent().ok_or(anyhow::anyhow!("No parent"))?).await?;

    if !path.exists() { // fetching data
        let data = client.get(url).send().await?.bytes().await?;
        tokio::fs::write(path, data).await?;
    }

    Ok(())
}

pub trait Cache:DeserializeOwned{

    type AcceptStorePoint:BaseStorePoint;

    /// this will check file exist or not.
    /// if the file exist, it will return the data from disk.
    /// if the file not exist, it will fetch the data from the source and save it to the disk, then return the data.
    /// a dirty way to avoid async trait warning, you should see this as `` async fn try_cache -> anyhow::Result<Self>; ``
    fn try_cache<P: AsRef<Path>+Send>(base:&Self::AcceptStorePoint, suffix:P, client: Client, url:&str)
        -> impl std::future::Future<Output = anyhow::Result<Self>> + Send{async move {

        let path = base.get_base().join(suffix);

        handle_file_not_exist(&path, &client, url).await?;

        let content = std::fs::read_to_string(path)?;
        let json = serde_json::from_str(&content)?;

        Ok(json)
    }}

    /// 1. the file exist and the sha is valid, return the data from disk.
    /// 2. the file exist and the sha is invalid, fetch the data from the source and save it to the disk, then return the data.
    /// 3. the file not exist, fetch the data from the source and save it to the disk, then return the data.
    fn check_cache<P: AsRef<Path>+Send>(base:&Self::AcceptStorePoint, suffix:P, client: Client, url: &str, sha:SHA)
        -> impl std::future::Future<Output = anyhow::Result<Self>> + Send{async move {

        let path = base.get_base().join(suffix);
        handle_file_not_exist(&path, &client, url).await?;

        let content = std::fs::read(path.clone())?;

        let valid = match sha {
            SHA::SHA1(a) => {
                let mut hasher = sha1::Sha1::new();
                hasher.update(&content);
                hasher.finalize().as_slice() == a
            }
            SHA::SHA256(b) => {
                let mut hasher = sha2::Sha256::new();
                hasher.update(&content);
                hasher.finalize().as_slice() == b
            }
        };

        if !valid{
            match client.get(url).send().await?.bytes().await{
                Ok(data) => {tokio::fs::write(&path, data).await?;}
                Err(e) => {log::error!("Error while fetching {url}, details:{}",e.to_string())} // we won't do anything if the data is not fetched successfully.
            };
        }

        let content = std::fs::read_to_string(path)?; // we won't check the sha again, because we already download it.

        let json = serde_json::from_str(&content)?;
        Ok(json)
    }}

    /// Return a builder for the cache.
    fn builder() -> CacheBuilder<Self::AcceptStorePoint,Self> where Self::AcceptStorePoint:Clone{
        CacheBuilder{
            url:"".to_string(),
            buf:PathBuf::new(),
            base:None,
            _t: PhantomData,
        }
    }

}


/// Using builder pattern for [Cache] trait.
/// This builder is required [T] impl Clone trait to use.
pub struct CacheBuilder<T:BaseStorePoint,U:Cache>{
    url:String,
    buf:PathBuf,
    base:Option<T>,
    _t:PhantomData<U>
}


impl <T,U> CacheBuilder<T, U> where U:Cache<AcceptStorePoint=T>, T:BaseStorePoint+Clone{

    /// append the path to the buffer.
    pub fn add<P: AsRef<Path>+Send>(mut self,args:P) -> Self{
        self.buf.push(args);
        self
    }

    /// change the url you want to fetch.
    pub fn url<P: AsRef<str>>(mut self,args:P) -> Self{
        self.url = args.as_ref().to_string();
        self
    }

    /// set the base path of the data.
    pub fn base_on(mut self, args:&T) -> Self{
        self.base = Some(args.clone());
        self
    }

    /// run [U::check_cache] from builder and return the result.
    pub fn build_check(&self, client: Client, sha:SHA)
                       -> impl std::future::Future<Output=anyhow::Result<U>> + Send + '_{
        let base = &self.base.as_ref().unwrap();
        U::check_cache(base,&self.buf,client,&self.url,sha)
    }

    /// run [U::try_cache] from builder and return the result.
    pub fn build_try(&self, client: Client) -> impl std::future::Future<Output = anyhow::Result<U>> + Send + '_{
        let base = &self.base.as_ref().unwrap();
        U::try_cache(base,&self.buf,client,&self.url)
    }

}