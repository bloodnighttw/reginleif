use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub trait BaseStorePoint:Sync+Send{
    /// Get the path of the data.
    fn get_base(&self) -> PathBuf;
}

pub trait ExpandStorePoint{
    fn get_suffix(&self) -> PathBuf;
}

pub trait Store<'a>:Serialize+Deserialize<'a>{

    const FILE_PATH:&'static [&'static str];
    /// The type of the base path you have to accept.
    type AcceptStorePoint:BaseStorePoint;
    /// You should assign Self to this. 
    type SelfType;

    fn full_path(base:&Self::AcceptStorePoint) -> PathBuf{
        let mut base_path = base.get_base();
        for i in Self::FILE_PATH{
            base_path = base_path.join(i);
        }
        base_path
    }
    
    fn save(&self,base:&Self::AcceptStorePoint) -> anyhow::Result<()>;
    fn load(base:&Self::AcceptStorePoint) -> anyhow::Result<Self::SelfType>;
}

pub trait Save:ExpandStorePoint+Serialize{

    /// The type of the base path you have to accept.
    type AcceptStorePoint:BaseStorePoint;
    
    fn save(&self, save:Self::AcceptStorePoint) -> anyhow::Result<()>;
}

pub trait Load<'a>:ExpandStorePoint+Deserialize<'a>{

    /// The type of the base path you have to accept.
    type AcceptStorePoint:BaseStorePoint;
    
    fn load<P: AsRef<Path>>(base:Self::AcceptStorePoint,suffix:P) -> anyhow::Result<()>;
}