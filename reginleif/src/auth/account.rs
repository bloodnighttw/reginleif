use std::time::Duration;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::auth::microsoft::MicrosoftAuth;
use crate::auth::minecraft::{MinecraftAuth, Profile};
use crate::auth::xbox::{XboxLiveToken, XboxSecurityToken};
use crate::utils::expiring_data::{Expirable, ExpiringData, Refreshable};


/// Account
/// 
/// This struct is used to store the user's account information.
/// including Minecraft Auth, Profile, and Microsoft Auth.
/// 
/// This struct also impl [Expirable](crate::utils::expiring_data::Expirable) and [Refreshable](crate::utils::expiring_data::Refreshable) trait,
/// this means you can use it with ExpiringData<T> and you can refresh the account if the data is expired.
///
/// 
/// To use this struct, you need to create a new account with the data you get from the auth process.
/// # Example
/// ```no_run
/// use reqwest::Client;
/// use reginleif::auth::account::Account;
/// use reginleif::auth::microsoft::{DeviceCode, MicrosoftAuthError};
/// use reginleif::auth::minecraft::{MinecraftAuth, Profile};
///
///
/// use reginleif::auth::xbox::{XboxLiveToken, XboxSecurityToken};
///
///
/// async fn example(){
///        let client = reqwest::Client::new();
///        let client_id = "47f3e635-2886-4628-a1c2-fd8a9f4d7a5f";
///        let res = DeviceCode::fetch(&client,client_id).await;
///
///        let device_code = match res{
///            Ok(device_code) => {
///                println!("auth url: {}",device_code.verification_uri);
///                println!("user code: {}",device_code.user_code);
///                device_code
///            }
///            Err(e) => {
///                panic!("Error: {}",e); // error while fetching token.
///            }
///        };
///
///
///        let mut res = loop{
///            let result = device_code.exchange(&client,client_id).await;
///            let res = match result{
///                Ok(res) => {res}
///                Err(e) => {
///                    match e {
///                        MicrosoftAuthError::AuthorizationPending => {
///                            tokio::time::sleep(device_code.interval).await;
///                            continue;
///                        }
///                        _=> {panic!("Error: {}",e);}
///                    }
///                }
///            };
///            break res;
///        };
///
///        println!("{:?}",res);
///
///        let cloned = res.clone();
///
///        let xbox_live_token = XboxLiveToken::fetch(&client,&res.data.access_token).await.unwrap();
///        println!("{:?}",xbox_live_token);
///        let xbox_security_token = XboxSecurityToken::fetch(&client,xbox_live_token).await.unwrap();
///        println!("{:?}",xbox_security_token);
///        
///        let minecraft_auth = MinecraftAuth::fetch(&client,xbox_security_token).await.unwrap();
///        println!("{:?}",minecraft_auth);
///        let profile = Profile::fetch(&client,&minecraft_auth).await.unwrap();
///        println!("{:?}",profile);
///
///        let account:Account = (minecraft_auth,profile,res.clone()).into(); // convert into Account
///
/// }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Account{
    pub mc_auth:MinecraftAuth,
    pub profile:Profile,
    pub msa:ExpiringData<MicrosoftAuth>
}

impl Expirable for Account{
    fn get_duration(&self) -> Duration {
        self.mc_auth.expires_in
    }
}

#[async_trait]
impl Refreshable for Account{

    /// client id
    type Args = String;

    /// re-fetch minecraft auth and profile.
    async fn refresh(&mut self, client_id:&String) -> anyhow::Result<()> {

        let client = Client::new();

        let msa = &self.msa.try_ref(&(client.clone(),client_id.to_string())).await?;
        let xbox_live = XboxLiveToken::fetch(&client,&msa.access_token).await?;
        let xbox_security = XboxSecurityToken::fetch(&client,xbox_live).await?;
        let mc_auth = MinecraftAuth::fetch(&client,xbox_security).await?;
        let profile = Profile::fetch(&client,&mc_auth).await?;

        self.mc_auth = mc_auth;
        self.profile = profile;

        Ok(())
    }
}

impl From<(MinecraftAuth,Profile,ExpiringData<MicrosoftAuth>)> for Account{
    fn from((mc_auth,profile,msa): (MinecraftAuth, Profile, ExpiringData<MicrosoftAuth>)) -> Self {
        Self{
            mc_auth,
            profile,
            msa
        }
    }
}