mod test{
    use std::path::PathBuf;
    use serde::{Deserialize, Serialize};
    use reginleif_utils::save_path::{BaseStorePoint, Store};

    const TEST_PATH_CONST:&'static[&'static str] = &["test"];

    struct TestPath(PathBuf);

    impl BaseStorePoint for TestPath{
        fn get_base(&self) -> PathBuf {
            self.0.clone()
        }
    }

    impl From<PathBuf> for TestPath{
        fn from(path:PathBuf) -> Self{
            Self(path)
        }
    }

    #[derive(Deserialize,Serialize,PartialEq,Debug)]
    struct A;

    impl Store<'_> for A{
        const FILE_PATH: &'static [&'static str] = TEST_PATH_CONST;
        type AcceptStorePoint = TestPath;
        type SelfType = Self;

        fn save(&self, base: &Self::AcceptStorePoint) -> anyhow::Result<()> {
            let base_path = Self::full_path(&base);

            std::fs::create_dir_all(base_path.parent().ok_or(anyhow::anyhow!("No parent"))?)?;
            std::fs::write(base_path,serde_json::to_string(self)?.as_bytes())?;

            Ok(())

        }

        fn load(base: &Self::AcceptStorePoint) -> anyhow::Result<Self> {

            let base_path = Self::full_path(&base);

            let json = std::fs::read_to_string(base_path)?;
            Ok(serde_json::from_str(&json)?)
        }

    }

    #[tokio::test]
    async fn test_save_load(){
        let path = PathBuf::from("test");
        let test_path = TestPath::from(path.clone());
        let a = A;
        a.save(&test_path).unwrap();
        let b = A::load(&test_path).unwrap();
        assert_eq!(a,b);
        
        tokio::fs::remove_file(path).await.unwrap();
    }



}