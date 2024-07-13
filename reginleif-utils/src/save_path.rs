use std::path::{Path, PathBuf};
use serde::{Serialize};
use serde::de::DeserializeOwned;

pub trait BaseStorePoint:Sync+Send{
    /// Get the path of the data.
    fn get_base(&self) -> PathBuf;
}

pub trait ExpandStorePoint{
    fn get_suffix(&self) -> PathBuf;
}

pub trait Store:Serialize+DeserializeOwned{

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

    fn save(&self, base: &Self::AcceptStorePoint) -> anyhow::Result<()> {
        let base_path = Self::full_path(&base);

        std::fs::create_dir_all(base_path.parent().ok_or(anyhow::anyhow!("No parent"))?)?;
        std::fs::write(base_path,serde_json::to_string(self)?.as_bytes())?;

        Ok(())

    }

    fn load(base: &Self::AcceptStorePoint) -> anyhow::Result<Self> {

        let base_path = Self::full_path(&base);

        let json = std::fs::read_to_string(base_path)?;
        Ok(serde_json::from_str(&json)?)
    }
}

pub trait Save:ExpandStorePoint+Serialize{

    /// The type of the base path you have to accept.
    type AcceptStorePoint:BaseStorePoint;

    fn save(&self, base:&Self::AcceptStorePoint) -> anyhow::Result<()>{
        let base_path = base.get_base().join(&self.get_suffix());

        std::fs::create_dir_all(base_path.parent().ok_or(anyhow::anyhow!("No parent"))?)?;
        std::fs::write(base_path,serde_json::to_string(self)?.as_bytes())?;

        Ok(())
    }
}

pub trait Load:DeserializeOwned{

    /// The type of the base path you have to accept.
    type AcceptStorePoint:BaseStorePoint;
    type SelfType:DeserializeOwned;

    fn load<P: AsRef<Path>>(base: &Self::AcceptStorePoint, suffix: P) -> anyhow::Result<Self::SelfType>{
        let path = base.get_base().join(suffix);
        let content = std::fs::read_to_string(path)?;
        // Remove the explicit lifetime annotation from the call to `serde_json::from_str`
        let json = serde_json::from_str::<Self::SelfType>(&content)?;
        Ok(json)
    }
}