use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, Fields, FieldsNamed, FieldsUnnamed};

use crate::attrs::{parse_struct_attributes, OuterConfig, StructConfig};
use crate::error::{MacroError, MacroResult};
use crate::fields::{read_named_fields, read_unnamed_fields, write_named_fields, write_unnamed_fields};

//
// Reads
/////////////////////////////

pub(crate) fn struct_read(ident: &Ident, attrs: &Vec<Attribute>, fields: Fields) -> MacroResult {
    let config = parse_struct_attributes(attrs)?;

    match fields {
        Fields::Named(named_fields) => struct_read_fixed(named_fields, config),
        Fields::Unnamed(unnamed_fields) => tuple_struct_read_fixed(unnamed_fields, config),
        Fields::Unit => Err(MacroError::new(
            "Cannot derive ReadFixed for unit type",
            ident.span(),
        )),
    }
}

fn tuple_struct_read_fixed(fields: FieldsUnnamed, outer: StructConfig) -> MacroResult {
    let outer: OuterConfig = outer.into();
    let (names, reads) = read_unnamed_fields(&fields, &outer)?;

    let fun = quote! {
        fn read_fixed<R: std::io::Read>(buf: &mut R) -> Result<Self, fixcol::error::Error> {
            use fixcol::FixedDeserializer;
            #( #reads )*

            Ok(Self(#(#names),*))
        }
    };

    Ok(fun)
}

fn struct_read_fixed(fields: FieldsNamed, outer: StructConfig) -> MacroResult {
    let outer: OuterConfig = outer.into();
    let (field_names, field_reads) = read_named_fields(&fields, outer)?;

    let function = quote! {
        fn read_fixed<R: std::io::Read>(buf: &mut R) -> Result<Self, fixcol::error::Error> {
            use fixcol::FixedDeserializer;
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

pub(crate) fn struct_write(ident: &Ident, attrs: &Vec<Attribute>, fields: Fields) -> MacroResult {
    let config = parse_struct_attributes(attrs)?;

    let writes = match fields {
        Fields::Named(named_fields) => struct_write_fixed(named_fields, config)?,
        Fields::Unnamed(unnamed_fields) => tuple_struct_write_fixed(unnamed_fields, config)?,
        Fields::Unit => Err(MacroError::new(
            "Cannot derive WriteFixed for unit structs.",
            ident.span(),
        ))?,
    };

    Ok(writes)
}

fn struct_write_fixed(fields: FieldsNamed, config: StructConfig) -> MacroResult {
    let (names, configs) = write_named_fields(&fields, &OuterConfig::Struct(config))?;

    let gen = quote! {
        fn write_fixed<W: std::io::Write>(&self, buf: &mut W) -> Result<(), fixcol::error::Error> {
            use fixcol::FixedSerializer;

            #( let _ = self.#names.write_fixed_field(buf, #configs)?; )*

            Ok(())
        }
    };

    Ok(gen)
}

fn tuple_struct_write_fixed(fields: FieldsUnnamed, config: StructConfig) -> MacroResult {
    let (names, configs) = write_unnamed_fields(&fields, &OuterConfig::Struct(config))?;

    let gen = quote! {
        fn write_fixed<W: std::io::Write>(&self, buf: &mut W) -> Result<(), fixcol::error::Error> {
            use fixcol::FixedSerializer;

            #( let _ = self.#names.write_fixed_field(buf, #configs)?; )*

            Ok(())
        }
    };

    Ok(gen)
}
