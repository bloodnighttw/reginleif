use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use serde_json::Value::Object;

/// This enum is used to store the library information, it contains the common library
/// information or maven-based library information.
#[derive(Debug,Clone,Deserialize,PartialEq)]
#[serde(untagged)]
pub enum Library{
    Common(CommonLibrary),
    Maven(MavenLibrary)
}

/// This struct is used to store the maven-based library information.
#[derive(Debug,Clone,Deserialize,PartialEq)]
pub struct MavenLibrary{
    pub name:String,
    pub url:String,
}

/// This struct is used to store the common library information.
#[derive(Debug,Clone,Deserialize,PartialEq)]
pub struct CommonLibrary {
    pub name:String,
    pub downloads:Download,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub rules:Vec<Rule>,
    pub extract:Option<Extract>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub natives:HashMap<String,String>
}

/// This struct is used to store the rule of a library, which contain the information about
/// the package is need to install on the specific platform or not.
#[derive(Debug,Clone,Deserialize,PartialEq)]
pub struct Rule{
    action:Action,
    #[serde(deserialize_with = "os_processing")]
    os:Option<Platform>
}

/// This enum list all supported platforms.
#[derive(Debug,Clone,Deserialize,PartialEq)]
pub enum Platform{
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "windows-arm64")]
    WindowsArm64,
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "linux-arm32")]
    LinuxArm32,
    #[serde(rename = "linux-arm64")]
    LinuxArm64,
    #[serde(rename = "osx-arm64")]
    MacOsArm64,
    Unknown
}

impl Platform{
    pub fn me() -> Self{

        #[cfg(target_os = "windows")]
        #[cfg(target_arch = "x86_64")]
        return Self::Windows;

        #[cfg(target_os = "windows")]
        #[cfg((target_arch = "aarch64"))]
        return Self::WindowsArm64;

        #[cfg(target_os = "linux")]
        #[cfg(target_arch = "x86_64")]
        return Self::Linux;

        #[cfg(target_os = "linux")]
        #[cfg(target_arch = "arm")]
        return Self::LinuxArm32;

        #[cfg(target_os = "linux")]
        #[cfg(target_arch = "aarch64")]
        return Self::LinuxArm64;

        #[cfg(target_os = "macos")]
        #[cfg(target_arch = "aarch64")]
        return Self::MacOsArm64;

    }

    pub fn allow_rule(&self,rules:Vec<Rule>)->bool{
        if rules.len() == 0 {
            return true;
        }
        
        let mut data = false;

        for i in rules{
            match i.action {
                Action::Allow => {
                    if i.os.is_none() || Some(self.clone()) == i.os{
                        data = true   
                    }
                }
                Action::Disallow => {
                    if i.os.is_none() || Some(self.clone()) == i.os{
                        return false // disallow action has higher priority than allow
                    }
                }
            }
        }
        
        data
    }
}

impl From<&String> for Platform{
    fn from(value: &String) -> Self {
        match value.as_str(){
            "windows" => Platform::Windows,
            "windows-arm64" => Platform::WindowsArm64,
            "linux" => Platform::Linux,
            "linux-arm32" => Platform::LinuxArm32,
            "linux-arm64" => Platform::LinuxArm64,
            "osx-arm64" => Platform::MacOsArm64,
            _ => Platform::Unknown
        }
    }
}

/// Allow mean this rule is allow on the rule's platform, disallow mean this rule is disallow on the rule's platform.
#[derive(Debug,Clone,Deserialize,PartialEq)]
pub enum Action{
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "disallow")]
    Disallow
}

/// This struct is used to store the download information of a library or client.
/// It contains the artifact information or classifiers information, classifiers
/// is used to store some platform-specific libraries.
#[derive(Debug,Clone,Deserialize,PartialEq)]
pub struct Download{
    pub artifact:Option<Artifact>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub classifiers:HashMap<String,Artifact>,
}

/// This struct is used to store the artifact information of a library or client.
#[derive(Debug,Clone,Deserialize,PartialEq)]
pub struct Artifact{
    pub url:String,
    pub size:i64,
    pub sha1:String,
}

/// This struct is used to store the extract information of a library.
#[derive(Debug,Clone,Deserialize,PartialEq,Hash,Eq)]
pub struct Extract{
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    exclude:Vec<String>
}


/// This function is used to deserialize the os field in Rule struct.
fn os_processing<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Platform>, D::Error> {

    let obj = match Value::deserialize(deserializer)?{
        Object(obj) => {obj},
        _ => { unreachable!("Failed to deserialize the os field in Rule struct. This reached the unreachable code.")}
    };

    if let Some(Value::String(os)) = obj.get("name"){
        return Ok(Some(os.into()))
    }

    unreachable!("Failed to deserialize the os field in Rule struct. This reached the unreachable code.")

}