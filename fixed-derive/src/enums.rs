use proc_macro2::{token_stream, TokenStream};
use syn::{punctuated::Punctuated, token::Token, Attribute, FieldsNamed, FieldsUnnamed, Ident, Variant};
use syn::token::Comma;
use syn::Field;

use crate::attrs::{self, parse_variant_attributes, VariantConfig};

//
// Reads
//////////////////////////


pub(crate) fn enum_read(name: &Ident, attrs: &Vec<Attribute>, variants: Vec<&Variant>) 
    -> proc_macro2::TokenStream 
{
    let enum_config = attrs::parse_enum_attributes(attrs);

    let var: Vec<_> = variants.iter().map(|variant| {
        let var_name = &variant.ident;

        let VariantConfig { key } = parse_variant_attributes(&variant.attrs);

        match &variant.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => 
                read_struct_variant(key, var_name, named),
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => 
                read_tuple_variant(key, var_name, unnamed),
            syn::Fields::Unit => 
                read_unit_variant(key, var_name),
        }
    }).collect();

    unreachable!();
}

fn read_struct_variant(
    key: String,
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    unimplemented!("Do not yet support struct variants")
}

fn read_tuple_variant(
    key: String,
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    unimplemented!("Do not yet support tuple variants")
}

fn read_unit_variant(
    key: String,
    name: &Ident,
) -> TokenStream {
    unimplemented!("Do not yet support struct variants")
}



//
// Writes
//////////////////////////