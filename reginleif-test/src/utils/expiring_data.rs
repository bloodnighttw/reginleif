#[cfg(test)]
mod test{
    use std::time::Duration;
    use reginleif_macro::{Expirable, NoRefresh};
    use reginleif_utils::expiring_data::{ExpiringData, Refreshable};

    #[derive(Expirable,NoRefresh,Default)]
    struct TestStruct1{
        #[dur] duration: Duration
    }

    #[derive(Expirable,Default)]
    struct TestStruct2{
        #[dur] duration: Duration
    }

    #[async_trait::async_trait]
    impl Refreshable for TestStruct2{

        type Args = ();

        async fn refresh(&mut self,_:&()) -> anyhow::Result<()> {
            Ok(()) // do nothing in test
        }
    }

    #[tokio::test]
    pub async fn test_expire(){
        let test:ExpiringData<_> = TestStruct1::default().into();
        tokio::time::sleep(Duration::from_secs(2)).await;
        let temp = test.is_expired();
        let _test = test.get_ref();
        assert!(temp)
    }

    #[tokio::test]
    #[should_panic]
    pub async fn test_no_refresh(){
        let mut test:ExpiringData<_> = TestStruct1::default().into();
        test.refresh(&()).await.expect("it's should be panic!");
    }

    #[tokio::test]
    pub async fn test_expire2(){
        let mut test:ExpiringData<_> = TestStruct2::default().into();
        tokio::time::sleep(Duration::from_secs(2)).await;
        test.refresh(&()).await.unwrap();
        test.try_ref(&()).await.unwrap();
    }

}