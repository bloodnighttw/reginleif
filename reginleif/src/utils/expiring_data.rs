use std::time::Duration;
use chrono::Local;
use serde::{Deserialize, Serialize};
use crate::utils::serde_convert::{local_to_string, string_to_local};
use anyhow::Result;
use async_trait::async_trait;

/// This struct is used to store data that will expire.
///
/// # Attribute
///
/// * `data`: the data that will expire, the data must impl [Expirable](Expirable) and [Refreshable](Refreshable).
/// * `created_at`: the time this data created/refresh.
///
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ExpiringData<T> where T:Expirable+Refreshable
{

    /// the data that will expire
    pub data: T,
    /// the time this object created/refresh
    #[serde(deserialize_with = "string_to_local", serialize_with = "local_to_string")]
    pub created_at: chrono::DateTime<Local>,

}

/// A trait for data that can expire.
///
/// The data must have duration and the function `get_duration` must return the duration of the data.
/// You can also use ``Expirable`` derive and annotate the duration field
/// with ``#[dur]`` to annotate the duration field.
///
/// # Example
/// ```ignore
/// use std::time::Duration;
/// use reginleif_macro::Expirable;
///
/// #[derive(Expirable,Default)]
/// struct TestStruct1{
///     #[dur] duration: Duration
/// }
///
/// ```
pub trait Expirable{
    
    /// Get the duration of the data.
    ///
    /// Use in the [is_expired](ExpiringData::is_expired) function.
    fn get_duration(&self) -> Duration;

}

/// A trait for data that can be refreshed.
///
/// Note that all [ExpiringData](ExpiringData)'s data must impl this trait,
/// and if you want not to make it refreshable, you can just panic! or just return Ok(()) to it.
/// or you can use ``NoRefresh`` derive to make it panic when call this function.
///
/// # Example
/// ```ignore
/// use std::time::Duration;
/// use reginleif_macro::{Expirable, NoRefresh};
///
/// #[derive(Expirable,NoRefresh,Default)]
/// struct TestStruct1{
///     #[dur] duration: Duration
/// }
///
/// ```
/// 
/// You also need to know, since this is an async trait, it **should be used with async_trait crate macro**.
/// 
/// # Example
/// ```ignore
///  use std::time::Duration;
///  use reginleif::utils::expiring_data::Refreshable;
///  use reginleif_macro::Expirable;
///
///  #[derive(Expirable,Default)]
///  struct TestStruct2{
///     #[dur] duration: Duration
///  }
///
///  #[async_trait::async_trait] // you can also import it.
///  impl Refreshable for TestStruct2{
///     type Args = ();
///
///     async fn refresh(&mut self,_:&()) -> anyhow::Result<()> {
///         Ok(()) // do nothing in test
///     }
///  }
/// ``` 
#[async_trait]
pub trait Refreshable{
    type Args;
    /// Refresh the data.
    /// if the data don't have want to refresh, just not impl it,
    /// and it will panic when call this function.
    ///
    /// # Arguments
    /// * `args`: the arguments that need to refresh the data, if you don't know what to put, just use `&()`.
    async fn refresh(&mut self,args:&Self::Args) -> Result<()>;
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
    ///
    /// The function won't check the data is expired or not.
    pub fn get_ref(&self) -> &T{
        &self.data
    }
    
    /// Refresh data and update `created_at`
    pub async fn refresh(&mut self,args:&T::Args) -> Result<()>{
        self.data.refresh(args).await?;
        self.created_at = Local::now();
        Ok(())
    }
    
    /// Check the data is valid and return the reference of data.
    pub async fn try_ref(&mut self,args:&T::Args) -> Result<&T>{
        if self.is_expired(){
            self.refresh(args).await?;
        }
        Ok(&self.data)
    }

}


impl<T> From<T> for ExpiringData<T> where T:Expirable + Refreshable{
    /// Convert the data to `ExpiringData`.
    ///
    /// The `created_at` will be set to `Local::now()`.
    /// Note this means the data is created at the time of the function call,
    /// and **YOU SHOULD ENSURE** the data is valid when using this trait.
    fn from(data: T) -> Self{
        Self{
            data,
            created_at: Local::now(),
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

    #[async_trait::async_trait]
    impl Refreshable for TestStruct2{
        
        type Args = ();
        
        async fn refresh(&mut self,_:&()) -> anyhow::Result<()> {
            Ok(()) // do nothing in test
        }
    }

    #[tokio::test]
    pub async fn test_expire(){
        let test:ExpiringData<_> = TestStruct1::default().into();
        tokio::time::sleep(Duration::from_secs(2)).await;
        let temp = test.is_expired();
        let _test = test.get_ref();
        assert!(temp)
    }

    #[tokio::test]
    #[should_panic]
    pub async fn test_no_refresh(){
        let mut test:ExpiringData<_> = TestStruct1::default().into();
        test.refresh(&()).await.expect("it's should be panic!");
    }

    #[tokio::test]
    pub async fn test_expire2(){
        let mut test:ExpiringData<_> = TestStruct2::default().into();
        tokio::time::sleep(Duration::from_secs(2)).await;
        test.refresh(&()).await.unwrap();
        test.try_ref(&()).await.unwrap();
    }
    
}

