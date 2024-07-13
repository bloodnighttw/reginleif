use proc_macro::TokenStream;
use syn::{DeriveInput, ItemStruct, Meta};

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
        impl reginleif_utils::expiring_data::Expirable for #ident{
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
        impl reginleif_utils::expiring_data::Refreshable for #ident{
            
            type Args = ();

            async fn refresh(&mut self, args: &()) -> anyhow::Result<()> {
                panic!("The data struct can't use refresh function.");
            }
        }
    };
    
    token.into()
}

#[proc_macro_derive(BaseStorePoint)]
pub fn base_store_point(item:TokenStream) -> TokenStream{
    let ast:ItemStruct = syn::parse(item).unwrap();
    let ident = ast.ident;
    let token = quote::quote! {
        impl reginleif_utils::save_path::BaseStorePoint for #ident{
            fn get_base(&self) -> std::path::PathBuf {
                self.0.clone()
            }
        }
    };

    token.into()
}

fn impl_storage(ast:DeriveInput) -> TokenStream{
    let ident = ast.ident;
    let attr1 = ast.attrs.iter().filter(
        |x| x.path().is_ident("base_on")
    ).nth(0).expect("required #[base_on(BaseStorePoint)] to use this derive!");

    let attr1 = match &attr1.meta {
        Meta::List(a) => a.tokens.clone(),
        _o=> panic!("error while parsing argument!")
    };

    let attr2 = ast.attrs.iter().filter(
        |x| x.path().is_ident("filepath")
    ).nth(0).expect("required #[filepath(&'static [&static str])] to use this derive!");

    let attr2 = match &attr2.meta {
        Meta::List(a) => a.tokens.clone(),
        _o=> panic!("error while parsing argument!")
    };

    let token = quote::quote! {
        impl reginleif_utils::save_path::Store<'_> for #ident{
            const FILE_PATH: &'static [&'static str] = #attr2;
            type AcceptStorePoint = #attr1;
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
    };

    token.into()
}

#[proc_macro_derive(Storage, attributes(base_on,filepath))]
pub fn storage(item: TokenStream) -> TokenStream {
    let ast:DeriveInput = syn::parse(item).unwrap();
    let implement = impl_storage(ast);
    implement
}

fn impl_save(ast: DeriveInput) -> TokenStream{
    let ident = ast.ident;
    let attr1 = ast.attrs.iter().filter(
        |x| x.path().is_ident("base_on")
    ).nth(0).expect("required #[base_on(BaseStorePoint)] to use this derive!");

    let attr1 = match &attr1.meta {
        Meta::List(a) => a.tokens.clone(),
        _o=> panic!("error while parsing argument!")
    };

    let token = quote::quote! {
        impl reginleif_utils::save_path::Save for #ident{
            type AcceptStorePoint = #attr1;
        }
    };

    token.into()

}

#[proc_macro_derive(Save, attributes(base_on))]
pub fn save(item: TokenStream) -> TokenStream {
    let ast:DeriveInput = syn::parse(item).unwrap();
    let implement = impl_save(ast);
    implement
}

fn impl_load(ast: DeriveInput) -> TokenStream{
    let ident = ast.ident;
    let attr1 = ast.attrs.iter().filter(
        |x| x.path().is_ident("base_on")
    ).nth(0).expect("required #[base_on(BaseStorePoint)] to use this derive!");

    let attr1 = match &attr1.meta {
        Meta::List(a) => a.tokens.clone(),
        _o=> panic!("error while parsing argument!")
    };

    let token = quote::quote! {
        impl reginleif_utils::save_path::Load<'_> for #ident{
            type AcceptStorePoint = #attr1;
            type SelfType = Self;
        }
    };

    token.into()

}


#[proc_macro_derive(Load, attributes(base_on))]
pub fn load(item: TokenStream) -> TokenStream {
    let ast:DeriveInput = syn::parse(item).unwrap();
    let implement = impl_load(ast);
    implement
}