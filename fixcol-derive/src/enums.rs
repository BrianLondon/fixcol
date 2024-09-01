use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Attribute, FieldsNamed, FieldsUnnamed, Ident, Variant};

use crate::attrs::{
    fixcol_attrs, parse_enum_attributes, parse_variant_attributes, OuterConfig, VariantConfig,
};
use crate::error::{MacroError, MacroResult};
use crate::fields::{
    read_named_fields, read_unnamed_fields, write_named_fields, write_unnamed_fields,
};

//
// Reads
//////////////////////////

pub(crate) fn enum_read(
    name: &Ident,
    attrs: &[Attribute],
    variants: Vec<&Variant>,
) -> MacroResult {
    let enum_config = parse_enum_attributes(name, attrs)?;

    let items: Result<Vec<(String, TokenStream)>, MacroError> = variants
        .iter()
        .map(|variant| -> Result<(String, TokenStream), MacroError> {
            let var_name = &variant.ident;

            let config: VariantConfig =
                parse_variant_attributes(var_name, &variant.attrs, &enum_config)?;
            let key = config.key.clone();

            let read = match &variant.fields {
                syn::Fields::Named(fields) => read_struct_variant(var_name, fields, config.into())?,
                syn::Fields::Unnamed(fields) if config.embed => {
                    read_embedded_variant(var_name, fields)?
                }
                syn::Fields::Unnamed(fields) => {
                    read_tuple_variant(var_name, fields, &config.into())?
                }
                syn::Fields::Unit => read_unit_variant(var_name),
            };

            Ok((key, read))
        })
        .collect(); // TODO: Gather all the errors instead of just the first

    let (var_name, var_read): (Vec<String>, Vec<TokenStream>) = items?.into_iter().unzip();

    let key_width = enum_config.key_width;

    let fun = quote! {
        fn read_fixed<R: std::io::Read>(buf: &mut R) -> Result<Self, fixcol::error::Error> {
            use fixcol::FixedDeserializer;

            let mut s: [u8; #key_width] = [0; #key_width];
            buf.read_exact(&mut s).map_err(|e| fixcol::error::Error::from(e))?;
            let key: String = String::from_utf8(s.to_vec())
                .map_err(|e| fixcol::error::Error::from(e))?;

            match key.as_str() {
                #(#var_name => { #var_read },)*
                k => Err(fixcol::error::Error::unknown_key_error(k.to_owned())),
            }
        }
    };

    Ok(fun)
}

fn read_struct_variant(name: &Ident, fields: &FieldsNamed, outer: OuterConfig) -> MacroResult {
    let (field_names, field_reads) = read_named_fields(fields, outer)?;

    let read_code = quote! {
        #(#field_reads)*
        Ok(Self::#name { #(#field_names),* })
    };

    Ok(read_code)
}

// TODO: figure out how to do strict cascade into embedded variants
// deleted: "outer: OuterConfig" from the function arguments
fn read_embedded_variant(name: &Ident, fields: &FieldsUnnamed) -> MacroResult {
    if fields.unnamed.len() != 1 {
        return Err(MacroError::new(
            "Embed param is only valid on variants with exactly one field",
            fields.span(),
        ));
    }
    if let Some(field) = fields.unnamed.first() {
        if let Some(fa) = fixcol_attrs(&field.attrs).first() {
            return Err(MacroError::new(
                "Did not expect fixcol attribute on embedded enum variant",
                fa.meta.path().span(),
            ));
        }

        let inner_type = field.ty.clone();

        let code = quote! {
            let elem = #inner_type::read_fixed(buf)?;
            Ok(Self::#name(elem))
        };

        Ok(code)
    } else {
        unreachable!();
    }
}

fn read_tuple_variant(
    // key: String,
    name: &Ident,
    fields: &FieldsUnnamed,
    outer: &OuterConfig,
) -> MacroResult {
    let (field_labels, field_reads) = read_unnamed_fields(fields, outer)?;

    Ok(quote! {
        #(#field_reads)*
        Ok(Self::#name(#(#field_labels),*))
    })
}

fn read_unit_variant(
    // key: String,
    name: &Ident,
) -> TokenStream {
    quote! {
        Ok(Self::#name)
    }
}

//
// Writes
//////////////////////////

pub(crate) fn enum_write(
    name: &Ident,
    attrs: &[Attribute],
    variants: &[&Variant],
) -> MacroResult {
    let enum_config = parse_enum_attributes(name, attrs)?;

    let write_variants: Result<Vec<TokenStream>, MacroError> = variants
        .iter()
        .map(|variant| -> MacroResult {
            let config: VariantConfig =
                parse_variant_attributes(&variant.ident, &variant.attrs, &enum_config).unwrap(); // TODO: need to do this for write macros also

            let out = match &variant.fields {
                syn::Fields::Named(fields) => {
                    write_struct_variant(&variant.ident, &config, fields)?
                }
                syn::Fields::Unnamed(fields) if config.embed => {
                    write_embedded_variant(&variant.ident, &config, fields)?
                }
                syn::Fields::Unnamed(fields) => {
                    write_tuple_variant(&variant.ident, &config, fields)?
                }
                syn::Fields::Unit => write_unit_variant(&variant.ident, &config),
            };

            Ok(out)
        })
        .collect();

    let write_variants = write_variants?;

    let code = quote! {
        fn write_fixed<W: std::io::Write>(&self, buf: &mut W) -> Result<(), fixcol::error::Error> {
            use fixcol::FixedSerializer;

            match self {
                #(#write_variants)*
            }

            Ok(())
        }
    };

    Ok(code)
}

fn write_struct_variant(
    ident: &Ident,
    config: &VariantConfig,
    fields: &FieldsNamed,
) -> MacroResult {
    let key = config.key.to_owned();
    let key_len = key.len();
    let (names, configs) = write_named_fields(fields, &(*config).clone().into())?;

    // TODO: we may want to inherit strict for the key from the enum or variant
    let code = quote! {
        Self::#ident { #(#names),* } => {
            let key_config = fixcol::FieldDescription {
                skip: 0,
                len: #key_len,
                alignment: fixcol::Alignment::Left,
                strict: false,
            };
            let key = String::from(#key);
            let _ = key.write_fixed_field(buf, &key_config)?;

            #( let _ = #names.write_fixed_field(buf, #configs)?;  )*
        },
    };

    Ok(code)
}

fn write_tuple_variant(
    ident: &Ident,
    config: &VariantConfig,
    fields: &FieldsUnnamed,
) -> MacroResult {
    let (_, configs) = write_unnamed_fields(fields, &config.clone().into())?;
    let VariantConfig { key, strict, .. } = config;

    let named_fields: Vec<Ident> = configs
        .iter()
        .enumerate()
        .map(|f| format_ident!("f_{}", f.0))
        .collect();

    let key_len = key.len();

    // TODO: we may want to inherit strict for the key from the enum or variant
    let code = quote! {
        Self::#ident(#(#named_fields),*) => {
            let key_config = fixcol::FieldDescription {
                skip: 0,
                len: #key_len,
                alignment: fixcol::Alignment::Left,
                strict: #strict,
            };
            let key = String::from(#key);
            let _ = key.write_fixed_field(buf, &key_config)?;

            #( let _ = #named_fields.write_fixed_field(buf, #configs)?;  )*
        },
    };

    Ok(code)
}

fn write_embedded_variant(
    ident: &Ident,
    config: &VariantConfig,
    fields: &FieldsUnnamed,
) -> MacroResult {
    if fields.unnamed.len() != 1 {
        return Err(MacroError::new(
            "Embed param is only valid on variants with exactly one field",
            fields.span(),
        ));
    }

    if let Some(field) = fields.unnamed.first() {
        if let Some(fa) = fixcol_attrs(&field.attrs).first() {
            return Err(MacroError::new(
                "Did not expect fixcol attribute on embedded enum variant",
                fa.meta.path().span(),
            ));
        }

        let key_len = config.key.len();
        let key = config.key.clone();

        // TODO: we may want to inherit strict for the key from the enum or variant
        let gen = quote! {
            Self::#ident(inner) => {
                let key_config = fixcol::FieldDescription {
                    skip: 0,
                    len: #key_len,
                    alignment: fixcol::Alignment::Left,
                    strict: false,
                };
                let key = String::from(#key);
                let _ = key.write_fixed_field(buf, &key_config)?;

                inner.write_fixed(buf)?;
            }
        };

        Ok(gen)
    } else {
        unreachable!();
    }
}

fn write_unit_variant(ident: &Ident, config: &VariantConfig) -> TokenStream {
    let VariantConfig { key, .. } = config;
    let key_len = key.len();

    // TODO: we may want to inherit strict for the key from the enum or variant
    quote! {
        Self::#ident => {
            let key_config = fixcol::FieldDescription {
                skip: 0,
                len: #key_len,
                alignment: fixcol::Alignment::Left,
                strict: false,
            };
            let key = String::from(#key);
            let _ = key.write_fixed_field(buf, &key_config)?;
        },
    }
}
