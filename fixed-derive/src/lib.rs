extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use syn::{Data, DataStruct, DeriveInput};
use quote::quote;


#[proc_macro_derive(WriteFixed, attributes(fixed))]
pub fn write_fixed_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let function_impl = match ast.data {
        Data::Struct(DataStruct { fields: _, .. }) => quote!{
            fn write_fixed(&self, buf: &mut dyn std::io::Write) -> Result<(), ()> {
                let _ = buf.write("Foo".as_bytes());
                Ok(())
            }
        },
        Data::Enum(_) => panic!("Deriving ReadFixed on enums is not supported"),
        Data::Union(_) => panic!("Deriving ReadFixed on unions is not supported"),
    };

    let gen = quote! {
        impl #impl_generics crate::WriteFixed for #name #ty_generics #where_clause {
            #function_impl
        }
    };

    println!("{}", gen);

    gen.into()
}
