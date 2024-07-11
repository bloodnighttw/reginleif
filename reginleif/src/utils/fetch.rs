use async_trait::async_trait;
use reqwest::Client;


/// This trait is intended to be used with async function.
/// It will fetch data from the internet.
#[async_trait]
pub trait Fetch<T,R> {
    async fn fetch(&self,client:&Client,args:&T) -> R;
}