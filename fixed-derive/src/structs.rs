use crate::attrs::{self, FieldConfig};

use quote::{format_ident, quote};
use syn::{Fields, FieldsNamed, FieldsUnnamed};


//
// Reads
/////////////////////////////

pub(crate) fn struct_read(fields: Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(named_fields) => struct_read_fields(named_fields),
        Fields::Unnamed(unnamed_fields) => tuple_struct_read_fields(unnamed_fields),
        Fields::Unit => panic!("Cannot deserialize type with no inner data"),
    }
}

fn tuple_struct_read_fields(fields: syn::FieldsUnnamed) -> proc_macro2::TokenStream {
    let field_reads = fields.unnamed.iter().enumerate().map(|item| {
        let (field_num, field) = item;

        let ident = format_ident!("f{}", field_num);

        let config = attrs::parse_attributes(&field.attrs);
        let FieldConfig { skip, width, align: _ } = config;

        let buf_size = skip + width;

        // TODO: we shouldn't need a String here at all
        let read = quote! {
            let mut s: [u8; #buf_size] = [0; #buf_size];
            buf.read_exact(&mut s).map_err(|e| fixed::error::Error::from(e))?;
            let #ident = std::str::from_utf8(&s)
                .map_err(|e| fixed::error::Error::from_utf8_error(&s, e))?
                .parse_with(#config)
                .map_err(|e| fixed::error::Error::from(e))?;
        };

        (ident, read)
    });

    let (names, reads): (Vec<proc_macro2::Ident>, Vec<proc_macro2::TokenStream>) = field_reads.unzip();

    quote! {
        fn read_fixed<R: std::io::Read>(buf: &mut R) -> Result<Self, fixed::error::Error> {
            use fixed::FixedDeserializer;
            #( #reads )*

            Ok(Self(#(#names),*))
        }
    }
}

fn struct_read_fields(fields: syn::FieldsNamed) -> proc_macro2::TokenStream {
    let field_reads = fields.named.iter().map(|field| {
        let name = field.ident.as_ref().unwrap().clone();

        let config = attrs::parse_attributes(&field.attrs);
        let FieldConfig { skip, width, align: _ } = config;

        let buf_size = skip + width;

        // TODO: we shouldn't need a String here at all
        quote! {
            let mut s: [u8; #buf_size] = [0; #buf_size];
            buf.read_exact(&mut s).map_err(|e| fixed::error::Error::from(e))?;
            let #name = std::str::from_utf8(&s)
                .map_err(|e| fixed::error::Error::from_utf8_error(&s, e))?
                .parse_with(#config)
                .map_err(|e| fixed::error::Error::from(e))?;
        }
    });

    let field_names = fields.named.iter().map(|f| f.ident.clone());

    quote! {
        fn read_fixed<R: std::io::Read>(buf: &mut R) -> Result<Self, fixed::error::Error> {
            use fixed::FixedDeserializer;
            #(#field_reads)*

            Ok(Self {
                #(#field_names),*
            })
        }
    }
}

//
// Writes
///////////////////////////////////

pub(crate) fn struct_write(fields: Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(named_fields) => write_named_fields(named_fields),
        Fields::Unnamed(unnamed_fields) => write_unnamed_fields(unnamed_fields),
        Fields::Unit => 
            panic!("Unit structs not supported. Cannot serialize data type that hold no data"),
    }
}


fn write_named_fields(fields: FieldsNamed) -> proc_macro2::TokenStream {
    let field_writes = fields.named.iter().map(|field| {
        let name = field.ident.as_ref().unwrap().clone();
        let config = attrs::parse_attributes(&field.attrs);

        quote! {
            let _ = self.#name.write_fixed(
                buf,
                #config
            ).unwrap();
        }
    });

    quote! {
        fn write_fixed<W: std::io::Write>(&self, buf: &mut W) -> Result<(), ()> {
            use fixed::FixedSerializer;

            #( #field_writes )*

            Ok(())
        }
    }
}


fn write_unnamed_fields(fields: FieldsUnnamed) -> proc_macro2::TokenStream {
    let field_writes = fields.unnamed.iter().enumerate().map(|f| {
        let (num, field) = f;
        let name = syn::Index::from(num);
        let config = attrs::parse_attributes(&field.attrs);

        quote! {
            let _ = self.#name.write_fixed(
                buf,
                #config
            ).unwrap();
        }
    });

    quote! {
        fn write_fixed<W: std::io::Write>(&self, buf: &mut W) -> Result<(), ()> {
            use fixed::FixedSerializer;

            #( #field_writes )*

            Ok(())
        }
    }
}