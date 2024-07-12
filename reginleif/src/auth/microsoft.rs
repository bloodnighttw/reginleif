use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::Value;
use reginleif_macro::{Expirable, NoRefresh};
use crate::utils::serde_convert::{duration_to_sec, sec_to_duration};
use thiserror::Error;
use crate::auth::constant::{DEVICECODE_URL, GRANT_TYPE, REFRESH_GRANT_TYPE, SCOPE, TOKEN_URL};
use crate::utils::expiring_data::{ExpiringData, Refreshable};

/// The struct contain all the information that oauth2 device code auth flow need.
/// 
/// Note this struct is not intended to refresh.
/// If you want to get a valid token, you should abandon this and create new one directly, not using refresh to make it valid.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Expirable, NoRefresh)]
pub struct DeviceCode{

    /// A short string shown to the user used to identify the session on a secondary device.
    ///
    /// see [Microsoft DOCS](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#device-authorization-response)
    pub user_code: String,

    /// A long string used to verify the session between the client and the authorization server.
    /// The client uses this parameter to request the access token from the authorization server.
    ///
    /// see [Microsoft DOCS](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#device-authorization-response)
    pub device_code: String,

    /// The URI the user should go to with the user_code in order to sign in.
    ///
    /// see [Microsoft DOCS](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#device-authorization-response)
    pub verification_uri: String,

    /// The number of seconds before the device_code and user_code expire.
    ///
    /// see [Microsoft DOCS](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#device-authorization-response)
    #[serde(deserialize_with = "sec_to_duration", serialize_with = "duration_to_sec")]
    #[dur]
    pub expires_in: Duration,

    /// The number of seconds the client should wait between polling requests.
    ///
    /// see [Microsoft DOCS](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#device-authorization-response)
    #[serde(deserialize_with = "sec_to_duration", serialize_with = "duration_to_sec")]
    pub interval: Duration,
}

#[derive(Error,Debug)]
pub enum MicrosoftAuthError{

    /// if you get this error, you should try again in ``interval`` sec later.
    ///
    /// see [Microsoft DOCS](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#expected-errors)
    #[error("Failed to exchange device code. please try again. details:AuthorizationPending")]
    AuthorizationPending,

    /// User declined the authorization.
    ///
    /// see [Microsoft DOCS](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#expected-errors)
    #[error("Failed to exchange device code. details: AuthorizationDeclined")]
    AuthorizationDeclined,

    /// The auth server cannot identify the device code. Please check the device code is correct or not!
    ///
    /// see [Microsoft DOCS](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#expected-errors)
    #[error("Failed to exchange device code. details: BadVerificationCode")]
    BadVerificationCode,

    /// The token is expired.
    ///
    /// see [Microsoft DOCS](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#expected-errors)
    #[error("Failed to exchange device code. details: ExpiredToken")]
    ExpiredToken,

    /// This error is related to the request error.
    ///
    /// You can check error details in the inner error.
    #[error("Error while fetching token. details:{0}")]
    ReqwesetError(#[from] reqwest::Error),

    /// The error which is not on the list.
    ///
    /// You can check error details in the inner string.
    #[error("Unknown Error. details:{0}")]
    Others(String),
}


impl DeviceCode{

    /// To fetch the device code from the auth server.
    ///
    /// # Arguments
    /// * `client`: The reqwest client.
    /// * `client_id`: The client id of your app.
    ///
    /// # Returns
    /// * Return anyhow::Result<DeviceCode>
    ///
    /// # Example
    /// ```
    /// use reqwest::Client;
    /// use reginleif::auth::microsoft::DeviceCode;
    ///
    /// #[tokio::main]
    /// async fn main(){
    ///     let client = Client::new();
    ///     let client_id = "your_client_id";
    ///     let res = DeviceCode::fetch(&client,client_id).await;
    ///
    ///     match res{
    ///         Ok(device_code) => {
    ///             println!("auth url: {}",device_code.verification_uri);
    ///             println!("user code: {}",device_code.user_code);
    ///         }
    ///         Err(e) => {
    ///             panic!("Error: {}",e); // error while fetching token.
    ///         }
    ///     };
    /// }
    /// ```
    pub async fn fetch(client: &Client, client_id: &str) -> anyhow::Result<Self>{
        let params = HashMap::from([
            ("client_id", client_id),
            ("scope", SCOPE)
        ]);
        let res = client.post(DEVICECODE_URL)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json::<DeviceCode>()
            .await?;
        Ok(res)
    }

    /// To exchange the device code to a valid token.
    ///
    /// # Arguments
    /// * `client`: The reqwest client.
    /// * `client_id`: The client id of your app.
    ///
    /// # Returns
    /// * Return Result<MicrosoftAuthResponse,MicrosoftAuthError>
    ///
    /// # Example
    /// ```
    /// use reqwest::Client;
    /// use reginleif::auth::microsoft::DeviceCode;
    /// use reginleif::auth::microsoft::{MicrosoftAuthError, MicrosoftAuth};
    ///
    /// #[tokio::main]
    /// async fn main(){
    /// let client = Client::new();
    ///     let client_id = "your_client_id";
    ///     let res = DeviceCode::fetch(&client,client_id).await;
    ///
    ///     let device_code = match res{
    ///         Ok(device_code) => {
    ///             println!("auth url: {}",device_code.verification_uri);
    ///             println!("user code: {}",device_code.user_code);
    ///             device_code
    ///         }
    ///         Err(e) => {
    ///             panic!("Error: {}",e); // error while fetching token.
    ///         }
    ///     };
    ///
    ///     let res = loop{
    ///         let result = device_code.exchange(&client,client_id).await;
    ///         let res = match result{
    ///             Ok(res) => {res}
    ///             Err(e) => {
    ///                 match e {
    ///                     MicrosoftAuthError::AuthorizationPending => {
    ///                         tokio::time::sleep(device_code.interval).await;
    ///                         continue;
    ///                     }
    ///                     _=> {panic!("Error: {}",e);}
    ///                 }
    ///
    ///             }
    ///         };
    ///         break res;
    ///     };
    ///
    /// }
    /// ```
    pub async fn exchange(&self, client: &Client, client_id:&str) -> Result<ExpiringData<MicrosoftAuth>,MicrosoftAuthError>{

        let params = HashMap::from([
            (String::from("client_id"), client_id.to_string()),
            (String::from("grant_type"), String::from(GRANT_TYPE)),
            (String::from("device_code"), self.device_code.to_string()),
        ]);

        let res = client.post(TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(res.json::<MicrosoftAuth>().await?.into())
        } else {
            match res.json::<Value>()
                .await?
                .get("error")
                .ok_or(MicrosoftAuthError::Others("Error while reading error field".to_string()))?
                .as_str()
                .ok_or(MicrosoftAuthError::Others("Error while convert error field to str".to_string()))?
            {
                "authorization_pending" => Err(MicrosoftAuthError::AuthorizationPending),
                "authorization_declined" => Err(MicrosoftAuthError::AuthorizationDeclined),
                "bad_verification_code" => Err(MicrosoftAuthError::BadVerificationCode),
                "expired_token" => Err(MicrosoftAuthError::ExpiredToken),
                _other => Err(MicrosoftAuthError::Others(
                    format!("Unknown error: {:?}", _other)
                )),
            }
        }

    }

}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Expirable, PartialEq)]
pub struct MicrosoftAuth {
    pub token_type: String,
    pub scope: String,
    #[serde(deserialize_with = "sec_to_duration", serialize_with = "duration_to_sec")]
    #[dur]
    pub expires_in: Duration,
    pub access_token: String,
    pub refresh_token: String,
}

#[async_trait]
impl Refreshable for MicrosoftAuth{
    
    type Args = (Client,String);
    
    /// 
    /// 
    /// # Arguments 
    /// 
    /// * `args`: &(Client,String)
    ///     * Client: The reqwest client.
    ///     * String: The client id of your app.
    /// 
    /// returns: anyhow::Result<()> 
    /// 
    /// # Examples 
    /// 
    /// ```
    /// 
    /// use std::time::Duration; 
    /// use reginleif::auth::microsoft::MicrosoftAuthError;
    /// 
    /// #[tokio::main]
    /// async fn main(){
    ///
    ///  let client = reqwest::Client::new();    /// 
    ///  let client_id = "47f3e635-2886-4628-a1c2-fd8a9f4d7a5f";
    ///  let res = super::DeviceCode::fetch(&client,client_id).await;
    ///
    ///  let device_code = match res{
    ///     Ok(device_code) => { 
    ///         println!("auth url: {}",device_code.verification_uri);
    ///         println!("user code: {}",device_code.user_code);
    ///         device_code
    ///     }
    ///     Err(e) => {
    ///         panic!("Error: {}",e); // error while fetching token.
    ///     }
    ///  };
    ///
    ///
    ///  let mut res = loop{
    ///     let result = device_code.exchange(&client,client_id).await;
    ///     let res = match result{
    ///         Ok(res) => {res}
    ///         Err(e) => {
    ///             match e {
    ///                 MicrosoftAuthError::AuthorizationPending => {
    ///                     tokio::time::sleep(device_code.interval).await;
    ///                     continue;
    ///                 }
    ///             _=> {panic!("Error: {}",e);}
    ///             }
    ///         }   
    ///     };
    ///     break res;
    ///  };
    ///         
    ///         
    ///     let cloned = res.clone();
    ///         
    ///     tokio::time::sleep(Duration::from_secs(2)).await;
    ///     res.refresh(&(client,client_id.to_string())).await.unwrap();
    ///
    ///         
    ///  }
    /// ```
    async fn refresh(&mut self, args: &(Client,String)) -> anyhow::Result<()> {
        
        let (client,client_id) = args;

        let params = HashMap::from([
            (String::from("client_id"), client_id.to_string()),
            (String::from("grant_type"), String::from(REFRESH_GRANT_TYPE)),
            (String::from("scope"), SCOPE.to_string()),
            (String::from("refresh_token"), self.refresh_token.to_string()),
        ]);
        
        let mut data = client.post(TOKEN_URL)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        self = &mut data;
        Ok(())
    }
}
