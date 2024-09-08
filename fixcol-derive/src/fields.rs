use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{FieldsNamed, FieldsUnnamed, Index, Token, Type};

use crate::attrs::{self, parse_field_attributes, FieldConfig, OuterConfig};
use crate::error::MacroError;

fn add_turbo_to_type(path: &syn::TypePath) -> syn::TypePath {
    let mut new_path = path.clone();

    for segment in new_path.path.segments.iter_mut() {
        let span = segment.span();

        match &mut segment.arguments {
            syn::PathArguments::None => {}
            syn::PathArguments::Parenthesized(_) => {}
            syn::PathArguments::AngleBracketed(ref mut args) => {
                args.colon2_token = Some(Token![::](span));
            }
        }
    }

    new_path
}

pub(crate) fn read_unnamed_fields(
    fields: &FieldsUnnamed,
    outer_config: &OuterConfig,
) -> Result<(Vec<Ident>, Vec<TokenStream>), MacroError> {
    let last_field = fields.unnamed.len().saturating_sub(1);

    let field_reads: Result<Vec<(Ident, TokenStream)>, MacroError> = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|item| -> Result<(Ident, TokenStream), MacroError> {
            let (field_num, field) = item;

            let type_token = field.ty.clone();
            let ident = format_ident!("_{}", field_num);

            let config = attrs::parse_field_attributes(&item.1.span(), &field.attrs, outer_config)
                .map_err(|e| e.replace_span(field.span()))?;
            let FieldConfig { skip, width, strict, .. } = config;

            let buf_size = skip + width;

            let read_field = if field_num == last_field && !strict {
                quote! {
                    let n = buf.read(&mut s)
                        .map_err(|e| fixcol::error::Error::from(e))?;
                    let raw = String::from_utf8(s[..n].to_vec())
                        .map_err(|e| fixcol::error::Error::from(e))?;
                }
            } else {
                quote! {
                    buf.read_exact(&mut s)
                        .map_err(|e| fixcol::error::Error::from(e))?;
                    let raw = String::from_utf8(s.to_vec())
                        .map_err(|e| fixcol::error::Error::from(e))?;
                }
            };

            // TODO: we shouldn't need a String here at all
            let read = quote! {
                let mut s: [u8; #buf_size] = [0; #buf_size];
                #read_field
                let #ident = #type_token::parse_fixed(raw.as_str(), #config)
                    .map_err(|e| fixcol::error::Error::from(e))?;
            };

            Ok((ident, read))
        })
        .collect();

    Ok(field_reads?.into_iter().unzip())
}

/// Retuns field names and code to read those fields
pub(crate) fn read_named_fields(
    fields: &FieldsNamed,
    outer_config: OuterConfig,
) -> Result<(Vec<Ident>, Vec<TokenStream>), MacroError> {
    let last_field = fields.named.len().saturating_sub(1);

    let field_reads: Result<Vec<(Ident, TokenStream)>, MacroError> = fields
        .named
        .iter()
        .enumerate()
        .map(|item| -> Result<(Ident, TokenStream), MacroError> {
            let (field_num, field) = item;

            let type_token = match &field.ty {
                Type::Path(path) => Type::Path(add_turbo_to_type(path)),
                other => other.clone(),
            };
            let name = field.ident.as_ref().unwrap().clone();

            let config = parse_field_attributes(&name.span(), &field.attrs, &outer_config)?;
            let FieldConfig { skip, width, strict, .. } = config;

            let buf_size = skip + width;

            let read_field = if field_num == last_field && !strict {
                quote! {
                    let n = buf.read(&mut s)
                        .map_err(|e| fixcol::error::Error::from(e))?;
                    let raw = String::from_utf8(s[..n].to_vec())
                        .map_err(|e| fixcol::error::Error::from(e))?;
                }
            } else {
                quote! {
                    buf.read_exact(&mut s)
                        .map_err(|e| fixcol::error::Error::from(e))?;
                    let raw = String::from_utf8(s.to_vec())
                        .map_err(|e| fixcol::error::Error::from(e))?;
                }
            };

            // TODO: we shouldn't need a String here at all
            let read = quote! {
                let mut s: [u8; #buf_size] = [0; #buf_size];
                #read_field
                let #name = #type_token::parse_fixed(raw.as_str(), #config)
                    .map_err(|e| fixcol::error::Error::from(e))?;
            };

            Ok((name, read))
        })
        .collect();

    Ok(field_reads?.into_iter().unzip())
}

pub(crate) fn write_named_fields(
    fields: &FieldsNamed,
    outer_config: &OuterConfig,
) -> Result<(Vec<Ident>, Vec<FieldConfig>), MacroError> {
    let field_configs: Result<Vec<(Ident, FieldConfig)>, MacroError> = fields
        .named
        .iter()
        .map(|field| -> Result<(Ident, FieldConfig), MacroError> {
            let name = field.ident.as_ref().unwrap().clone();
            let config = attrs::parse_field_attributes(&name.span(), &field.attrs, outer_config)?;

            Ok((name, config))
        })
        .collect();

    Ok(field_configs?.into_iter().unzip())
}

pub(crate) fn write_unnamed_fields(
    fields: &FieldsUnnamed,
    outer_config: &OuterConfig,
) -> Result<(Vec<Index>, Vec<FieldConfig>), MacroError> {
    let field_configs: Result<Vec<(Index, FieldConfig)>, MacroError> = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|field| -> Result<(Index, FieldConfig), MacroError> {
            let name = syn::Index::from(field.0);
            let config =
                attrs::parse_field_attributes(&field.1.span(), &field.1.attrs, outer_config)?;

            Ok((name, config))
        })
        .collect();

    Ok(field_configs?.into_iter().unzip())
}

#[cfg(test)]
mod tests {
    use syn::TypePath;

    use super::*;

    #[test]
    fn add_turbo_where_needed() {
        let orig: TypePath = syn::parse_str("Option<u64>").unwrap();
        let actual = add_turbo_to_type(&orig);
        let expected: TypePath = syn::parse_str("Option::<u64>").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn dont_add_turbo_where_not_needed() {
        let orig: TypePath = syn::parse_str("Option::<u64>").unwrap();
        let actual = add_turbo_to_type(&orig);
        let expected: TypePath = syn::parse_str("Option::<u64>").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn qualified_turbo() {
        let orig: TypePath = syn::parse_str("custom::Result<u64>").unwrap();
        let actual = add_turbo_to_type(&orig);
        let expected: TypePath = syn::parse_str("custom::Result::<u64>").unwrap();
        assert_eq!(actual, expected);

        let orig: TypePath = syn::parse_str("custom::Result::<u64>").unwrap();
        let actual = add_turbo_to_type(&orig);
        let expected: TypePath = syn::parse_str("custom::Result::<u64>").unwrap();
        assert_eq!(actual, expected);
    }
}
