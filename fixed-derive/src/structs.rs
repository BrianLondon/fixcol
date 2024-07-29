use crate::{attrs::{self, FieldConfig}, fields::{read_named_fields, read_unnamed_fields}};

use quote::{format_ident, quote};
use syn::{Attribute, Fields, FieldsNamed, FieldsUnnamed, Ident};


//
// Reads
/////////////////////////////

pub(crate) fn struct_read(name: &Ident, attrs: &Vec<Attribute>, fields: Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(named_fields) => struct_read_fixed(named_fields),
        Fields::Unnamed(unnamed_fields) => tuple_struct_read_fixed(unnamed_fields),
        Fields::Unit => panic!("Cannot deserialize type with no inner data"),
    }
}

fn tuple_struct_read_fixed(fields: syn::FieldsUnnamed) -> proc_macro2::TokenStream {
    let (names, reads) = read_unnamed_fields(&fields);

    quote! {
        fn read_fixed<R: std::io::Read>(buf: &mut R) -> Result<Self, fixed::error::Error> {
            use fixed::FixedDeserializer;
            #( #reads )*

            Ok(Self(#(#names),*))
        }
    }
}

fn struct_read_fixed(fields: syn::FieldsNamed) -> proc_macro2::TokenStream {
    let (field_names, field_reads) = read_named_fields(&fields);

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
        let config = attrs::parse_field_attributes(&field.attrs);

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
        let config = attrs::parse_field_attributes(&field.attrs);

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