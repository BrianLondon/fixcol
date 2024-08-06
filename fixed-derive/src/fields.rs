use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{spanned::Spanned, FieldsNamed, FieldsUnnamed, Index};

// TODO: should FieldConfig live here? yes if it doesnt cause circular
use crate::{attrs::{self, FieldConfig}, error::MacroError};

pub(crate) fn read_unnamed_fields(
    fields: &FieldsUnnamed
) -> Result<(Vec<Ident>, Vec<TokenStream>), MacroError> {
    let field_reads: Result<Vec<(Ident ,TokenStream)>, MacroError> = fields.unnamed
        .iter().enumerate()
        .map(|item| -> Result<(Ident, TokenStream), MacroError> {
            let (field_num, field) = item;

            let type_token = field.ty.clone();
            let ident = format_ident!("_{}", field_num);

            let config = attrs::parse_field_attributes(&ident, &field.attrs)
                .map_err(|e| e.replace_span(field.span()))?;
            let FieldConfig { skip, width, align: _ } = config;

            let buf_size = skip + width;

            // TODO: we shouldn't need a String here at all
            let read = quote! {
                let mut s: [u8; #buf_size] = [0; #buf_size];
                buf.read_exact(&mut s).map_err(|e| fixed::error::Error::from(e))?;
                let raw = String::from_utf8(s.to_vec()).map_err(|e| fixed::error::Error::from(e))?;
                let #ident = #type_token::parse_fixed(raw.as_str(), #config)
                    .map_err(|e| fixed::error::Error::from(e))?;
            };

            Ok((ident, read))
        }).collect();
    
    Ok(field_reads?.into_iter().unzip())
}

/// Retuns field names and code to read those fields
pub(crate) fn read_named_fields(
    fields: &FieldsNamed
) -> Result<(Vec<Ident>, Vec<TokenStream>), MacroError> {
    let field_reads: Result<Vec<(Ident, TokenStream)>, MacroError> = fields.named
        .iter()
        .map(|field| -> Result<(Ident, TokenStream), MacroError> {
            let type_token = field.ty.clone();
            let name = field.ident.as_ref().unwrap().clone();

            let config = attrs::parse_field_attributes(&name, &field.attrs)?;
            let FieldConfig { skip, width, align: _ } = config;

            let buf_size = skip + width;

            // TODO: we shouldn't need a String here at all
            let read = quote! {
                let mut s: [u8; #buf_size] = [0; #buf_size];
                buf.read_exact(&mut s).map_err(|e| fixed::error::Error::from(e))?;
                let raw = String::from_utf8(s.to_vec()).map_err(|e| fixed::error::Error::from(e))?;
                let #name = #type_token::parse_fixed(raw.as_str(), #config)
                    .map_err(|e| fixed::error::Error::from(e))?;
            };

            Ok((name, read))
        })
        .collect();

    Ok(field_reads?.into_iter().unzip())
}

pub(crate) fn write_named_fields(
    fields: &FieldsNamed
) -> Result<(Vec<Ident>, Vec<FieldConfig>), MacroError> {
    let field_configs: Result<Vec<(Ident, FieldConfig)>, MacroError> = fields
        .named
        .iter()
        .map(|field| -> Result<(Ident, FieldConfig), MacroError> {
            let name = field.ident.as_ref().unwrap().clone();
            let config = attrs::parse_field_attributes(&name, &field.attrs)?;

            Ok((name, config))
        })
        .collect();

    Ok(field_configs?.into_iter().unzip())
}

pub(crate) fn write_unnamed_fields(
    fields: &FieldsUnnamed
) -> Result<(Vec<Index>, Vec<FieldConfig>), MacroError> {
    let field_configs: Result<Vec<(Index, FieldConfig)>, MacroError> = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|field| -> Result<(Index, FieldConfig), MacroError> {
            let name = syn::Index::from(field.0);
            let config = attrs::parse_field_attributes(
                &format_ident!("_{}", field.0), &field.1.attrs)?;

            Ok((name, config))
        })
        .collect();

    Ok(field_configs?.into_iter().unzip())
}
