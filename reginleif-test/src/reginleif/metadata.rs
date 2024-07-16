#[cfg(test)]
mod client{
    use std::path::PathBuf;
    use reginleif::metadata::client::package::PackageList;
    use reginleif_macro::BaseStorePoint;

    #[derive(Clone,Debug,BaseStorePoint)]
    struct TestPath(PathBuf);

    impl TestPath{
        pub fn new() -> Self{
            Self(PathBuf::from("test-metadata"))
        }
    }

    type TestPackageList = PackageList<TestPath>;

    #[tokio::test]
    async fn test_metadata_fetch()->anyhow::Result<()>{

        let client = reqwest::Client::new();
        let base_path = TestPath::new();
        let endpoint = "https://meta.prismlauncher.org/v1/";

        let test_package = [
            ("net.minecraft","1.14"),
            ("net.fabricmc.fabric-loader","0.16.0"),
        ];

        let packages = TestPackageList::fetch(&base_path,client.clone(),endpoint).await?;


        for (uid,version) in test_package{
            let pkg = packages.iter().find(|x| x.uid == uid).unwrap();
            let pkg_details = pkg.get_details(&base_path,client.clone(),endpoint).await.unwrap();
            // println!("{:?}",pkg_details);
            let version_info = pkg_details.iter().find(|x| x.version == version).unwrap();
            // println!("{:?}",version_info);
            let version_details = version_info.get_details(&base_path,client.clone(),endpoint,uid).await?;
            // println!("{:?}",version_details);
            if version_details.asset_index.is_some(){
                let assets = version_details.asset_index.unwrap().fetch_assets_info(&base_path,client.clone()).await?;
                // println!("{:?}",assets);
            }

        }

        Ok(())
    }


}