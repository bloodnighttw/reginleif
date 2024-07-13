#[cfg(test)]
mod test{
    use std::path::{Path, PathBuf};
    use serde::{Deserialize, Serialize};
    use reginleif_macro::{BaseStorePoint, Load, Save, Storage};
    use reginleif_utils::save_path::{BaseStorePoint, ExpandStorePoint, Load, Save, Store};


    #[derive(BaseStorePoint)]
    struct TestPath(PathBuf);

    impl From<PathBuf> for TestPath{
        fn from(path:PathBuf) -> Self{
            Self(path)
        }
    }

    #[derive(Deserialize,Serialize,PartialEq,Debug,Storage)]
    #[base_on(TestPath)] #[filepath(&["test.txt"])]
    struct A;

    #[tokio::test]
    async fn test_static_save_load(){
        let path = PathBuf::from("test");
        let test_path = TestPath::from(path.clone());
        let a = A;
        a.save(&test_path).unwrap();
        let b = A::load(&test_path).unwrap();
        assert_eq!(a,b);

        tokio::fs::remove_dir_all(path).await.unwrap();
    }

    #[derive(Serialize,Deserialize,Save,Load,PartialEq,Debug)]
    #[base_on(TestPath)]
    struct B;

    impl ExpandStorePoint for B{
        fn get_suffix(&self) -> PathBuf {
            PathBuf::from("test223.txt")
        }
    }

    #[tokio::test]
    async fn test_dynamic_save_load(){
        let path = PathBuf::from("test");
        let test_path = TestPath::from(path.clone());
        let b = B;
        b.save(&test_path).unwrap();

        let temp = B::load(&test_path,"test223.txt").unwrap();
        assert_eq!(b,temp);

    }

}