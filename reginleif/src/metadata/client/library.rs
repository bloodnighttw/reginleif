use std::collections::HashMap;
use serde::Deserialize;

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
