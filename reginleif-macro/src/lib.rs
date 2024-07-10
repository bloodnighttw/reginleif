use proc_macro::TokenStream;
use syn::ItemStruct;

fn impl_expire(ast:ItemStruct) -> TokenStream{
    let ident = ast.ident;
    let duration_filed = ast.fields.iter()
        .find(
            |x| -> bool {
                let has_attr = x.attrs.iter().find(
                    |i| i.path().segments.len() == 1 && i.path().segments[0].ident == "dur"
                );
                has_attr.is_some()
            }
        ).expect("you should have attr #[dur] in your std::time::Duration field");

    let ident2 = duration_filed.ident.clone().unwrap();

    let token = quote::quote! {
        impl crate::utils::expiring_data::Expirable for #ident{
            fn get_duration(&self) -> std::time::Duration {
                self.#ident2
            }
        }
    };

    token.into()
}

#[proc_macro_derive(Expirable, attributes(dur))]
pub fn time_sensitive(item:TokenStream) -> TokenStream{
    let ast:ItemStruct = syn::parse(item).unwrap();
    impl_expire(ast)
}


/// This function provide derive macro of refresh, it will make it panic!
#[proc_macro_derive(NoRefresh, attributes(dur))]
pub fn refresh_panic(item:TokenStream) -> TokenStream{
    let ast:ItemStruct = syn::parse(item).unwrap();
    let ident = ast.ident;
    let token = quote::quote! {
        #[async_trait::async_trait]
        impl crate::utils::expiring_data::Refreshable<()> for #ident{
            async fn refresh(&mut self,_:()) -> anyhow::Result<()>{
                panic!("The data struct can't use refresh function.");
            }
        }
    };
    
    token.into()
}
