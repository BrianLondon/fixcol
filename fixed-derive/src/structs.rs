use proc_macro2::Ident;
use quote::quote;
use syn::{Fields, FieldsNamed, FieldsUnnamed};

use crate::{error::{MacroError, MacroResult}, fields::{
    read_named_fields, read_unnamed_fields, write_named_fields, write_unnamed_fields,
}};

//
// Reads
/////////////////////////////

pub(crate) fn struct_read(
    ident: &Ident,
    fields: Fields,
) -> MacroResult {
    match fields {
        Fields::Named(named_fields) => struct_read_fixed(named_fields),
        Fields::Unnamed(unnamed_fields) => tuple_struct_read_fixed(unnamed_fields),
        Fields::Unit => {
            Err(MacroError::new(
                "Cannot deserialize type with no inner data",
                ident.span()
            ))
        }
    }
}

fn tuple_struct_read_fixed(fields: FieldsUnnamed) -> MacroResult {
    let (names, reads) = read_unnamed_fields(&fields)?;

    let fun = quote! {
        fn read_fixed<R: std::io::Read>(buf: &mut R) -> Result<Self, fixed::error::Error> {
            use fixed::FixedDeserializer;
            #( #reads )*

            Ok(Self(#(#names),*))
        }
    };

    Ok(fun)
}

fn struct_read_fixed(fields: FieldsNamed) -> MacroResult {
    let (field_names, field_reads) = read_named_fields(&fields)?;

   let function = quote! {
        fn read_fixed<R: std::io::Read>(buf: &mut R) -> Result<Self, fixed::error::Error> {
            use fixed::FixedDeserializer;
            #(#field_reads)*

            Ok(Self {
                #(#field_names),*
            })
        }
    };

    Ok(function)
}

//
// Writes
///////////////////////////////////

pub(crate) fn struct_write(fields: Fields) -> MacroResult {
    let writes = match fields {
        Fields::Named(named_fields) => struct_write_fixed(named_fields)?,
        Fields::Unnamed(unnamed_fields) => tuple_struct_write_fixed(unnamed_fields)?,
        Fields::Unit => {
            panic!("Unit structs not supported. Cannot serialize data types that hold no data")
        }
    };

    Ok(writes)
}

fn struct_write_fixed(fields: FieldsNamed) -> MacroResult {
    let (names, configs) = write_named_fields(&fields)?;

    let gen = quote! {
        fn write_fixed<W: std::io::Write>(&self, buf: &mut W) -> Result<(), fixed::error::Error> {
            use fixed::FixedSerializer;

            #( let _ = self.#names.write_fixed_field(buf, #configs)?; )*

            Ok(())
        }
    };

    Ok(gen)
}

fn tuple_struct_write_fixed(fields: FieldsUnnamed) -> MacroResult {
    let (names, configs) = write_unnamed_fields(&fields)?;

    let gen = quote! {
        fn write_fixed<W: std::io::Write>(&self, buf: &mut W) -> Result<(), fixed::error::Error> {
            use fixed::FixedSerializer;

            #( let _ = self.#names.write_fixed_field(buf, #configs)?;  )*

            Ok(())
        }
    };

    Ok(gen)
}
