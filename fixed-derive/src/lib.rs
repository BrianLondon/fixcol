mod attrs;
mod enums;
mod fields;
mod structs;

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use attrs::FieldConfig;
use enums::enum_write;
use proc_macro::TokenStream;

use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput};

use crate::enums::enum_read;
use crate::structs::{struct_read, struct_write};

// This doesn't really belong here, but there's not a better place
// it's function spans the arg parsing and code generating steps
// Maybe in fields.rs?
impl quote::ToTokens for FieldConfig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let FieldConfig { skip, width, align } = &self;

        let alignment = match &align {
            attrs::Align::Left => quote! { fixed::Alignment::Left },
            attrs::Align::Right => quote! { fixed::Alignment::Right },
            attrs::Align::Full => quote! { fixed::Alignment::Full },
        };

        tokens.extend(quote! {
            &fixed::FieldDescription {
                skip: #skip,
                len: #width,
                alignment: #alignment,
            }
        });
    }
}

#[proc_macro_derive(ReadFixed, attributes(fixed))]
pub fn read_fixed_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let attrs = &ast.attrs;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let function_impl = match ast.data {
        Data::Struct(DataStruct { fields, .. }) => struct_read(name, attrs, fields),
        Data::Enum(DataEnum { variants, .. }) => enum_read(name, attrs, variants.iter().collect()),
        Data::Union(_) => panic!("Deriving ReadFixed on unions is not supported"),
    };

    let gen = quote! {
        impl #impl_generics fixed::ReadFixed for #name #ty_generics #where_clause {
            #function_impl
        }
    };

    println!("{}", gen);

    gen.into()
}

#[proc_macro_derive(WriteFixed, attributes(fixed))]
pub fn write_fixed_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let function_impl = match ast.data {
        Data::Struct(DataStruct { fields, .. }) => struct_write(fields),
        Data::Enum(DataEnum { variants, .. }) => enum_write(variants.iter().collect()),
        Data::Union(_) => panic!("Deriving WriteFixed on unions is not supported"),
    };

    let gen = quote! {
        impl #impl_generics fixed::WriteFixed for #name #ty_generics #where_clause {
            #function_impl
        }
    };

    // println!("{}", gen);

    gen.into()
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn macro_test() {
        assert!(true);
    }
}
