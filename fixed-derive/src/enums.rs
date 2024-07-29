use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, FieldsNamed, FieldsUnnamed, Ident, Variant};

use crate::attrs::{self, parse_variant_attributes, VariantConfig};
use crate::fields::{read_named_fields, read_unnamed_fields};

//
// Reads
//////////////////////////


pub(crate) fn enum_read(
    name: &Ident, 
    attrs: &Vec<Attribute>, 
    variants: Vec<&Variant>
) -> proc_macro2::TokenStream {
    let enum_config = attrs::parse_enum_attributes(attrs);

    let (var_name, var_read): (Vec<_>, Vec<_>) = variants.iter().map(|variant| {
        let var_name = &variant.ident;

        let VariantConfig { key } = parse_variant_attributes(&variant.attrs);

        let read = match &variant.fields {
            syn::Fields::Named(fields) => 
                read_struct_variant(var_name, fields),
            syn::Fields::Unnamed(fields) => 
                read_tuple_variant(var_name, fields),
            syn::Fields::Unit => 
                read_unit_variant(var_name),
        };

        (key, read)
    }).unzip();

    let key_width = enum_config.key_width;

    quote! {
        fn read_fixed<R: std::io::Read>(buf: &mut R) -> Result<Self, fixed::error::Error> {
            use fixed::FixedDeserializer;

            let mut s: [u8; #key_width] = [0; #key_width];
            buf.read_exact(&mut s).map_err(|e| fixed::error::Error::from(e))?;
            let key: &str = std::str::from_utf8(&s)
                .map_err(|e| fixed::error::Error::from_utf8_error(&s, e))?;

            match key {
                #(#var_name => { #var_read },)*
                k => Err(fixed::error::Error::unknown_key_error(k.to_owned())),
            }
        }

    }
}

fn read_struct_variant(
    // key: String,
    name: &Ident,
    fields: &FieldsNamed,
) -> TokenStream {
    let (field_names, field_reads) = read_named_fields(fields);

    quote! {
        #(#field_reads)*
        Ok(Self::#name { #(#field_names),* })
    }
}

fn read_tuple_variant(
    // key: String,
    name: &Ident,
    fields: &FieldsUnnamed,
) -> TokenStream {
    let (field_labels, field_reads) = read_unnamed_fields(fields);

    quote! {
        #(#field_reads)*
        Ok(Self::#name(#(#field_labels),* ))
    }
}

fn read_unit_variant(
    // key: String,
    name: &Ident,
) -> TokenStream {
    unimplemented!("Do not yet support unit variants")
}



//
// Writes
//////////////////////////