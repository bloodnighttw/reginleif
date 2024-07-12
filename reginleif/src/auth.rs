pub mod microsoft;
pub mod xbox;
pub mod minecraft;
pub mod account;
mod constant;

#[cfg(test)]
mod test{
    use std::time::Duration;
    use crate::auth::account::Account;
    use crate::auth::microsoft::{DeviceCode, MicrosoftAuthError};
    use crate::auth::minecraft::{MinecraftAuth, Profile};
    use crate::auth::xbox::{XboxLiveToken, XboxSecurityToken};
    use crate::utils::expiring_data::ExpiringData;

    #[tokio::test]
    #[ignore]
    async fn test_auth_token(){

        let client = reqwest::Client::new();
        let client_id = "47f3e635-2886-4628-a1c2-fd8a9f4d7a5f";
        let res = DeviceCode::fetch(&client,client_id).await;

        let device_code = match res{
            Ok(device_code) => {
                println!("auth url: {}",device_code.verification_uri);
                println!("user code: {}",device_code.user_code);
                device_code
            }
            Err(e) => {
                panic!("Error: {}",e); // error while fetching token.
            }
        };


        let mut res = loop{
            let result = device_code.exchange(&client,client_id).await;
            let res = match result{
                Ok(res) => {res}
                Err(e) => {
                    match e {
                        MicrosoftAuthError::AuthorizationPending => {
                            tokio::time::sleep(device_code.interval).await;
                            continue;
                        }
                        _=> {panic!("Error: {}",e);}
                    }
                }
            };
            break res;
        };

        println!("{:?}",res);

        let cloned = res.clone();

        let xbox_live_token = XboxLiveToken::fetch(&client,&res.data.access_token).await.unwrap();
        println!("{:?}",xbox_live_token);
        let xbox_security_token = XboxSecurityToken::fetch(&client,xbox_live_token).await.unwrap();
        println!("{:?}",xbox_security_token);
        
        let minecraft_auth = MinecraftAuth::fetch(&client,xbox_security_token).await.unwrap();
        println!("{:?}",minecraft_auth);
        let profile = Profile::fetch(&client,&minecraft_auth).await.unwrap();
        println!("{:?}",profile);

        let account:Account = (minecraft_auth,profile,res.clone()).into();
        let mut account: ExpiringData<Account> = account.into();
        account.refresh(&client_id.to_string()).await.unwrap();
        println!("{:?}",account);

        tokio::time::sleep(Duration::from_secs(2)).await;
        res.refresh(&(client,client_id.to_string())).await.unwrap();

        assert_ne!(cloned.created_at,res.created_at);
        

    }

}