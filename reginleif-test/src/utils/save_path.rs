#[cfg(test)]
mod test{
    use std::marker::PhantomData;
    use std::path::{PathBuf};
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use reginleif_macro::{BaseStorePoint, Load, Save, Storage};
    use reginleif_utils::save_path::{BaseStorePoint, Cache, ExpandStorePoint, Load, Save, Store};
    use reginleif_utils::sha::SHA;

    #[derive(BaseStorePoint,PartialEq,Debug,Clone)]
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
        let path = PathBuf::from("test1");
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
        let path = PathBuf::from("test2");
        let test_path = TestPath::from(path.clone());
        let b = B;
        b.save(&test_path).unwrap();

        let temp = B::load(&test_path,"test223.txt").unwrap();
        assert_eq!(b,temp);

        tokio::fs::remove_dir_all(path).await.unwrap();

    }

    #[derive(Serialize,Deserialize,PartialEq,Debug)]
    struct C<T> where T:BaseStorePoint{
        num:String,
        _t:PhantomData<T>
    }

    impl <T> ExpandStorePoint for C<T> where T:BaseStorePoint{
        fn get_suffix(&self) -> PathBuf {
            PathBuf::from(&format!("{}.txt",&self.num))
        }
    }

    impl <T> Save for C<T> where T:BaseStorePoint{
        type AcceptStorePoint = T;
    }

    impl <T> Load for C<T> where T:BaseStorePoint{
        type AcceptStorePoint = T;
    }

    type D = C<TestPath>;

    impl From<String> for D{
        fn from(value: String) -> Self {
            Self{
                num: value,
                _t: Default::default(),
            }
        }
    }

    #[tokio::test]
    async fn generic_test(){
        let path = PathBuf::from("test3");
        let test_path = TestPath::from(path.clone());
        let d:D = String::from("123").into();
        d.save(&test_path).unwrap();

        let temp = D::load(&test_path,"123.txt").unwrap();
        assert_eq!(d,temp);

        tokio::fs::remove_dir_all(path).await.unwrap();
    }


    #[derive(Serialize,Deserialize,PartialEq,Debug,Save,Load)]
    struct E<T> where T:BaseStorePoint{
        num:String,
        _t:PhantomData<T>
    }

    impl <T> ExpandStorePoint for E<T> where T:BaseStorePoint{
        fn get_suffix(&self) -> PathBuf {
            PathBuf::from(&format!("{}.txt",&self.num))
        }
    }

    #[derive(Deserialize,Debug)]
    struct OUO{
        uid:String
    }

    impl Cache for OUO{
        type AcceptStorePoint = TestPath;
    }

    #[tokio::test]
    async fn cache_test() -> anyhow::Result<()>{
        let sha:SHA = "c0094ab29be4be93b7cf0e05067608814afb6c4f40223784ecb69e6635cd6bbf".try_into()?;

        let base: TestPath = PathBuf::from("test_ouo").into();
        let client = Client::new();

        let _a = OUO::builder()
            .base_on(&base)
            .url("https://meta.prismlauncher.org/v1/org.lwjgl/")
            .add("test.txt")
            .build_try(client.clone()).await?;

        let _b = OUO::builder()
            .base_on(&base)
            .url("https://meta.prismlauncher.org/v1/org.lwjgl/")
            .add("test.txt")
            .build_check(client.clone(),sha).await?;

        tokio::fs::remove_dir_all(PathBuf::from("test_ouo")).await?;
        Ok(())
    }


}