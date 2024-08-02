use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{FieldsNamed, FieldsUnnamed, Index};

// TODO: should FieldConfig live here? yes if it doesnt cause circular
use crate::attrs::{self, FieldConfig};

pub(crate) fn read_unnamed_fields(fields: &FieldsUnnamed) -> (Vec<Ident>, Vec<TokenStream>) {
    let field_reads = fields.unnamed.iter().enumerate().map(|item| {
        let (field_num, field) = item;

        let type_token = field.ty.clone();
        let ident = format_ident!("f{}", field_num);

        let config = attrs::parse_field_attributes(&ident, &field.attrs);
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

        (ident, read)
    });

    field_reads.unzip()
}

/// Retuns field names and code to read those fields
pub(crate) fn read_named_fields(fields: &FieldsNamed) -> (Vec<Ident>, Vec<TokenStream>) {
    let field_reads = fields.named.iter().map(|field| {
        let type_token = field.ty.clone();
        let name = field.ident.as_ref().unwrap().clone();

        let config = attrs::parse_field_attributes(&name, &field.attrs);
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

        (name, read)
    });

    field_reads.unzip()
}

pub(crate) fn write_named_fields(fields: &FieldsNamed) -> (Vec<Ident>, Vec<FieldConfig>) {
    fields
        .named
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap().clone();
            let config = attrs::parse_field_attributes(&name, &field.attrs);

            (name, config)
        })
        .unzip()
}

// TODO: replace f0, f1, etc with _0, _1, etc.
pub(crate) fn write_unnamed_fields(fields: &FieldsUnnamed) -> (Vec<Index>, Vec<FieldConfig>) {
    fields
        .unnamed
        .iter()
        .enumerate()
        .map(|field| {
            let name = syn::Index::from(field.0);
            let config = attrs::parse_field_attributes(&format_ident!("f{}", field.0), &field.1.attrs);

            (name, config)
        })
        .unzip()
}
