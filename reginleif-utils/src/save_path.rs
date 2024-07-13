//! The module for the save and load the data from the file.
//! It provides the trait to save and load the data from the file,
//! from the path that is constant to dynamic.
use std::path::{Path, PathBuf};
use serde::{Serialize};
use serde::de::DeserializeOwned;

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