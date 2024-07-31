use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, FieldsNamed, FieldsUnnamed, Ident, Variant};

use crate::attrs::{self, parse_variant_attributes, VariantConfig};
use crate::fields::{read_named_fields, read_unnamed_fields, write_named_fields, write_unnamed_fields};

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
        Ok(Self::#name(#(#field_labels),*))
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

pub(crate) fn enum_write(
    name: &Ident,
    attrs: &Vec<Attribute>, 
    variants: Vec<&Variant>
) -> proc_macro2::TokenStream {
    let write_variants = variants.iter().map(|variant| {
        let VariantConfig { key } = parse_variant_attributes(&variant.attrs);

        match &variant.fields {
            syn::Fields::Named(fields) => 
                write_struct_variant(&variant.ident, key, fields),
            syn::Fields::Unnamed(fields) => 
                write_tuple_variant(&variant.ident, key, fields),
            syn::Fields::Unit => write_unit_variant(&variant.ident, key),
        }
    });

    quote! {
        fn write_fixed<W: std::io::Write>(&self, buf: &mut W) -> Result<(), ()> {
            use fixed::FixedDeserializer;

            #(#write_variants)*
        }
    }    
}

fn write_struct_variant(
    ident: &Ident, 
    key: String, 
    fields: &FieldsNamed
) -> TokenStream {
    let (names, configs) = write_named_fields(&fields);

    let key_len = key.len();

    quote! {
        v @ Self::#ident {..} => {
            let key_config = fixed::FieldDescription {
                skip: 0,
                len: #key_len,
                alignment: Alignment::Left,
            };
            let key = String::from(#key);
            let _ = key.write_fixed(buf, key_config).unwrap();

            #( let _ = self.#names.write_fixed(buf, #configs).unwrap();  )*
        },
    }
}


fn write_tuple_variant(
    ident: &Ident, 
    key: String, 
    fields: &FieldsUnnamed
) -> TokenStream {
    let (ids, configs) = write_unnamed_fields(&fields);

    let key_len = key.len();

    quote! {
        v @ Self::#ident(..) => {
            let key_config = fixed::FieldDescription {
                skip: 0,
                len: #key_len,
                alignment: Alignment::Left,
            };
            let key = String::from(#key);
            let _ = key.write_fixed(buf, key_config).unwrap();

            #( let _ = self.#ids.write_fixed(buf, #configs).unwrap();  )*
        },
    }
}

fn write_unit_variant(ident: &Ident, key: String) -> TokenStream {
    let key_len = key.len();

    quote! {
        v @ Self::#ident(..) => {
            let key_config = fixed::FieldDescription {
                skip: 0,
                len: #key_len,
                alignment: Alignment::Left,
            };
            let key = String::from(#key);
            let _ = key.write_fixed(buf, key_config).unwrap();
        },
    }
}
