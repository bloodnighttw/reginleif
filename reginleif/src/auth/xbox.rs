use anyhow::anyhow;
use reqwest::Client;
use serde_json::{json, Value};
use thiserror::Error;
use crate::auth::constant::{XBOX_USER_AUTHENTICATE, XBOX_XSTS_AUTHORIZE};
use crate::auth::xbox::XboxSecurityError::Others;

/// Xbox Live Token
/// 
/// This token is used to authenticate with Xbox Security Token.
#[derive(Debug)]
pub struct XboxLiveToken(String);

impl From<&str> for XboxLiveToken {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<XboxLiveToken> for String {
    fn from(token: XboxLiveToken) -> Self {
        token.0
    }
}

impl XboxLiveToken {
    
    /// Fetch Xbox Live Token
    /// 
    /// This function will fetch Xbox Live Token from the given token.
    /// 
    /// # Arguments
    /// * `client` - The reqwest client
    /// * `token` - The microsoft auth token you get from [MicrosoftAuth](crate::auth::microsoft::MicrosoftAuth)
    /// 
    pub async fn fetch(client:&Client,token:&str) -> anyhow::Result<XboxLiveToken>{

        let xbox_authenticate_json = json!({
           "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": &format!("d={}", token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });

        let res = client.post(XBOX_USER_AUTHENTICATE)
            .json(&xbox_authenticate_json)
            .send()
            .await?
            .error_for_status()?;

        let token = res.json::<Value>()
            .await?
            .get("Token")
            .ok_or(anyhow!("Token not found"))?
            .as_str()
            .ok_or(anyhow!("Error while parsing token"))?
            .into();

        Ok(token)
    }
}

/// Xbox Security Token
/// 
/// This token is used to authenticate the user with Minecraft Auth Server.
#[derive(Debug)]
pub struct XboxSecurityToken{
    pub token: String,
    pub uhs: String,
}

impl From<(&str,&str)> for XboxSecurityToken{
    fn from((token,uhs): (&str, &str)) -> Self {
        Self{
            token: token.to_string(),
            uhs: uhs.to_string(),
        }
    }
}


/// The error that can occur while fetching Xbox Security Token.
#[derive(Error,Debug)]
pub enum XboxSecurityError{
    #[error("The account doesn't have an Xbox account. Once they sign up for one (or login through minecraft.net to create one) then they can proceed with the login")]
    NotExist,
    #[error("The account is banned from Xbox Live due to your country")]
    CountryBan,
    #[error("The account needs adult verification on Xbox page")]
    NeedAdultVerification,
    #[error("The account is a child (under 18) and cannot proceed unless the account is added to a Family by an adult.")]
    AddToFamily,
    #[error("Error while fetching Xbox Security Token. Details: {0}")]
    Others(String),
    #[error("Reqwest error. Details: {0}")]
    ReqwestError(reqwest::Error)

}

impl From<reqwest::Error> for XboxSecurityError{
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e)
    }
}

impl XboxSecurityToken{
    
    /// Fetch Xbox Security Token
    /// 
    /// This function will fetch Xbox Security Token from the given Xbox Live Token.
    /// 
    /// # Arguments
    /// * `client` - The reqwest client
    /// * `token` - The Xbox Live Token you get from [XboxLiveToken::fetch](XboxLiveToken::fetch)
    pub async fn fetch(client:&Client,token:XboxLiveToken) -> Result<XboxSecurityToken,XboxSecurityError> {

        let xbox_authenticate_json = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [token.0],
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });

        let response = client
            .post(XBOX_XSTS_AUTHORIZE)
            .json(&xbox_authenticate_json)
            .send()
            .await?;
        
        if response.status().is_success() {
            let value = response
                .json::<Value>()
                .await?;
            
            let token = value["Token"]
                .as_str()
                .ok_or(Others("Error while parsing token".to_string()))?;
            
            let user_hash = value["DisplayClaims"]["xui"][0]["uhs"]
                .as_str()
                .ok_or(Others("Error while parsing uhs".to_string()))?;

            Ok((token,user_hash).into())
        } else {

            let value = response
                .json::<Value>()
                .await?;

            match value["XErr"]
                .as_u64().ok_or(Others("Error while fetching XErr code".to_string()))? {
                2148916233 => Err(XboxSecurityError::NotExist),
                2148916235 => Err(XboxSecurityError::CountryBan),
                2148916236 | 2148916237 => {
                    Err(XboxSecurityError::NeedAdultVerification)
                }
                2148916238 => Err(XboxSecurityError::AddToFamily),
                _other => Err(Others(
                    "Unknown Error".to_string(),
                )),
            }

        }

    }
}


