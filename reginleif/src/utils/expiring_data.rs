use std::time::Duration;
use chrono::{Local};
use serde::{Deserialize, Serialize};
use crate::utils::serde_convert::{local_to_string, string_to_local};
use anyhow::Result;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ExpiringData<T> where T:Expirable{
    pub data: T,
    #[serde(deserialize_with = "string_to_local", serialize_with = "local_to_string")]
    pub created_at: chrono::DateTime<Local>,
}

/// A trait for data that can expire.
pub trait Expirable {
    
    /// Get the duration of the data.
    /// Use in the `is_expired` function.
    fn get_duration(&self) -> Duration;
    
    /// Refresh the data.
    /// if the data don't have want to refresh, just not impl it,
    /// and it will panic when call this function.
    async fn refresh(&mut self) -> Result<()>{
        panic!("The data struct can't use refresh function.");
    }
}

impl<T> ExpiringData<T>
where
    T: Expirable,
{
    
    /// Check the data is expired or not.
    pub fn is_expired(&self) -> bool{
        let duration = (Local::now() - self.created_at)
            .to_std()
            .expect("Failed to convert chrono::Duration to std::Duration");

        duration >= self.data.get_duration()
    }
    
    /// Get the reference of data.
    /// The function won't check the data is expired or not.
    pub fn get_ref(&self) -> &T{
        &self.data
    }
    
    /// Refresh data and update `created_at`
    pub async fn refresh(&mut self) -> Result<()>{
        self.data.refresh().await?;
        self.created_at = Local::now();
        Ok(())
    }
    
    /// Check the data is valid and return the reference of data.
    pub async fn try_ref(&mut self) -> Result<&T>{
        if self.is_expired(){
            self.refresh().await?;
        }
        Ok(&self.data)
    }

}

