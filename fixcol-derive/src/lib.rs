mod attrs;
mod enums;
mod error;
mod fields;
mod structs;

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use enums::enum_write;
use error::MacroError;
use proc_macro::TokenStream;

use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataEnum, DataStruct, DeriveInput};

use crate::enums::enum_read;
use crate::structs::{struct_read, struct_write};

/// Derive proc-macro for ReadFixed
//
// See documentation on [`ReadFixed`] for a full description.
//
// [`ReadFixed`]: fixcol::ReadFixed
#[proc_macro_derive(ReadFixed, attributes(fixcol))]
pub fn read_fixed_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let attrs = &ast.attrs;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let function_impl_result = match ast.data {
        Data::Struct(DataStruct { fields, .. }) => struct_read(&name, attrs, fields),
        Data::Enum(DataEnum { variants, .. }) => enum_read(name, attrs, variants.iter().collect()),
        Data::Union(u) => Err(MacroError::new(
            "Deriving ReadFixed on unions is not supported",
            u.union_token.span(),
        )),
    };

    let gen = match function_impl_result {
        Ok(function_impl) => {
            quote! {
                impl #impl_generics fixcol::ReadFixed for #name #ty_generics #where_clause {
                    #function_impl
                }
            }
        }
        Err(err) => quote! { #err },
    };

    // println!("{}", gen);

    gen.into()
}

/// Derive proc-macro for WriteFixed
//
// See [[`WriteFixed`]] for a complete discuassion.
#[proc_macro_derive(WriteFixed, attributes(fixcol))]
pub fn write_fixed_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let attrs = &ast.attrs;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let function_impl_result = match ast.data {
        Data::Struct(DataStruct { fields, .. }) => struct_write(name, attrs, fields),
        Data::Enum(DataEnum { variants, .. }) => {
            enum_write(name, attrs, &variants.iter().collect())
        }
        Data::Union(u) => Err(MacroError::new(
            "Deriving WriteFixed on unions is not supported",
            u.union_token.span(),
        )),
    };

    let gen = match function_impl_result {
        Ok(function_impl) => {
            quote! {
                impl #impl_generics fixcol::WriteFixed for #name #ty_generics #where_clause {
                    #function_impl
                }
            }
        }
        Err(err) => quote! { #err },
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
