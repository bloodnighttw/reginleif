use std::time::Duration;
use chrono::{Local};
use serde::{Deserialize, Serialize};
use crate::utils::serde_convert::{local_to_string, string_to_local};
use anyhow::Result;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ExpiringData<T> where T:Expirable+Refreshable{
    pub data: T,
    #[serde(deserialize_with = "string_to_local", serialize_with = "local_to_string")]
    pub created_at: chrono::DateTime<Local>,
}

/// A trait for data that can expire.
pub trait Expirable {
    
    /// Get the duration of the data.
    /// Use in the `is_expired` function.
    fn get_duration(&self) -> Duration;

}

pub trait Refreshable{
    /// Refresh the data.
    /// if the data don't have want to refresh, just not impl it,
    /// and it will panic when call this function.
    async fn refresh(&mut self) -> Result<()>;
}

impl<T> ExpiringData<T>
where
    T: Expirable + Refreshable,
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

impl<T> From<T> for ExpiringData<T> where T:Expirable + Refreshable{
    fn from(data: T) -> Self{
        Self{
            data,
            created_at: Local::now()
        }
    }
}

#[cfg(test)]
mod test{
    use std::time::Duration;
    use reginleif_macro::{Expirable, NoRefresh};
    use crate::utils::expiring_data::{ExpiringData, Refreshable};

    #[derive(Expirable,NoRefresh,Default)]
    struct TestStruct1{
        #[dur] duration: Duration
    }

    #[derive(Expirable,Default)]
    struct TestStruct2{
        #[dur] duration: Duration
    }

    impl Refreshable for TestStruct2{
        async fn refresh(&mut self) -> anyhow::Result<()> {
            Ok(()) // do nothing in test
        }
    }

    #[tokio::test]
    pub async fn test_expire(){
        let test:ExpiringData<_> = TestStruct1::default().into();
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(test.is_expired())
    }

    #[tokio::test]
    #[should_panic]
    pub async fn test_no_refresh(){
        let mut test:ExpiringData<_> = TestStruct1::default().into();
 
        test.refresh().await.expect("it's should be panic!");
    }

    #[tokio::test]
    pub async fn test_expire2(){
        let mut test:ExpiringData<_> = TestStruct2::default().into();
        tokio::time::sleep(Duration::from_secs(2)).await;
        test.refresh().await.unwrap();
        test.try_ref().await.unwrap();
    }
    
}

