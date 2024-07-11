use std::collections::HashMap;
use std::time::Duration;
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::Value;
use reginleif_macro::{Expirable, NoRefresh};
use crate::utils::serde_convert::{duration_to_sec, sec_to_duration};
use thiserror::Error;
use crate::auth::constant::{DEVICECODE_URL, GRANT_TYPE, SCOPE, TOKEN_URL};

/// The struct contain all the information that oauth2 device code auth flow need.
/// 
/// Note this struct is not intended to refresh.
/// If you want to get a valid token, you should abandon this and create new one directly, not using refresh to make it valid.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Expirable, NoRefresh)]
pub struct DeviceCode{
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    #[serde(deserialize_with = "sec_to_duration", serialize_with = "duration_to_sec")]
    #[dur]
    pub expires_in: Duration,
    #[serde(deserialize_with = "sec_to_duration", serialize_with = "duration_to_sec")]
    pub interval: Duration,
}

#[derive(Error,Debug)]
pub enum MicrosoftAuthError{
    #[error("Failed to exchange device code. please try again. details:AuthorizationPending")]
    AuthorizationPending,
    #[error("Failed to exchange device code. details: AuthorizationDeclined")]
    AuthorizationDeclined,
    #[error("Failed to exchange device code. details: BadVerificationCode")]
    BadVerificationCode,
    #[error("Failed to exchange device code. details: ExpiredToken")]
    ExpiredToken,
    #[error("Error while fetching token. details:{0}")]
    ReqwesetError(#[from] reqwest::Error),
    #[error("Unknown Error. details:{0}")]
    Others(String),
}


impl DeviceCode{
    pub async fn fetch(client: &Client, client_id: &str) -> anyhow::Result<Self>{
        let params = [
            ("client_id", client_id),
            ("scope", SCOPE)
        ];
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

    pub async fn exchange(&self, client: &Client, client_id:&str) -> anyhow::Result<MicrosoftAuthResponse,MicrosoftAuthError>{

        let params: HashMap<String, String> = HashMap::from([
            (String::from("client_id"), client_id.to_string()),
            (String::from("grant_type"), String::from(GRANT_TYPE)),
            (String::from("device_code"), self.device_code.to_string()),
        ]);

        let res = client.post(TOKEN_URL)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(res.json().await?)
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

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Expirable)]
pub struct MicrosoftAuthResponse {
    pub token_type: String,
    pub scope: String,
    #[serde(deserialize_with = "sec_to_duration", serialize_with = "duration_to_sec")]
    #[dur]
    pub expires_in: Duration,
    pub ext_expires_in: u64,
    pub access_token: String,
    pub refresh_token: String,
}

