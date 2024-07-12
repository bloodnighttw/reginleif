use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use reginleif_macro::{Expirable, NoRefresh};
use crate::auth::constant::{MINECRAFT_LOGIN_WITH_XBOX, MINECRAFT_PROFILE};
use crate::auth::xbox::XboxSecurityToken;
use crate::utils::serde_convert::{duration_to_sec, sec_to_duration};

/// Minecraft Auth
/// 
/// This struct is used to authenticate the user with Minecraft Auth Server.
#[derive(Serialize, Deserialize,Debug,Clone, Expirable, NoRefresh)]
pub struct MinecraftAuth {
    
    /// UUID of MinecraftAuth, note this is **NOT** the UUID of the player.
    pub username: String,
    /// The access token you can use to access Minecraft.
    pub access_token: String,
    
    /// The expires time of the access token.
    #[serde(deserialize_with = "sec_to_duration", serialize_with = "duration_to_sec")]
    #[dur]
    pub expires_in: Duration,
    /// The token type of the access token. Always being "bearer"
    pub token_type: String,
}

impl MinecraftAuth{
    
    /// Fetch Minecraft Auth
    /// 
    /// This function will fetch Minecraft Auth from the given Xbox Security Token.
    /// 
    /// # Arguments
    /// * `client` - The reqwest client
    /// * `xbox_security_token` - The Xbox Security Token you get from [XboxSecurityToken::fetch](crate::auth::xbox::XboxSecurityToken::fetch)
    pub async fn fetch(client: &Client, xbox_security_token: XboxSecurityToken) -> anyhow::Result<Self>{

        let res = client
            .post(MINECRAFT_LOGIN_WITH_XBOX)
            .header("Content-Type", "application/json")
            .json(&json!({
                "identityToken": format!("XBL3.0 x={};{}",xbox_security_token.uhs,xbox_security_token.token)
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(res)
    }
    
}

/// Minecraft Profile
/// 
/// This struct is used to store Minecraft Profile.If user doesn't have game, the profile won't exist too!
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    
    /// uuid of the player
    pub id: String,
    /// name of the player
    pub name: String,
    /// the skins of the player
    pub skins: Vec<Skin>,
    /// the capes of the player have
    pub capes: Vec<Caps>,
}

impl Profile{
    /// Fetch Minecraft Profile
    /// 
    /// This function will fetch Minecraft Profile from the given Minecraft Auth.
    /// If the user doesn't have game, the profile won't exist too!
    /// 
    /// # Arguments
    /// * `client` - The reqwest client
    /// * `microsoft_auth` - The Microsoft Auth you get from [MinecraftAuth::fetch](crate::auth::minecraft::MinecraftAuth::fetch)
    pub async fn fetch(client: &Client, microsoft_auth: &MinecraftAuth) -> anyhow::Result<Profile>{
        let res = client
            .get(MINECRAFT_PROFILE)
            .bearer_auth(&microsoft_auth.access_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(res)
    }
}


/// Minecraft Skin
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    /// the id of the skin
    pub id: String,
    /// ACTIVE or INACTIVE
    pub state: String,
    /// the url of the skin you can get.
    pub url: String,
    pub texture_key: String,
    /// Classic or Slim
    pub variant: String,
}

/// Minecraft Capes
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Caps {
    /// the id of the cape
    pub id: String,
    /// ACTIVE or INACTIVE
    pub state: String,
    /// the url of the cape you can get.
    pub url: String,
    /// the alias of the cape
    pub alias: String,
}