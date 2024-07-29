use proc_macro2::{Ident, TokenStream};
use quote::{quote, format_ident};
use syn::{FieldsNamed, FieldsUnnamed};

// TODO: should FieldConfig live here? yes if it doesnt cause circular
use crate::attrs::{self, FieldConfig};

pub(crate) fn read_unnamed_fields(
    fields: &FieldsUnnamed
) -> (Vec<Ident>, Vec<TokenStream>) {
    let field_reads = fields.unnamed.iter().enumerate().map(|item| {
        let (field_num, field) = item;

        let ident = format_ident!("f{}", field_num);

        let config = attrs::parse_field_attributes(&field.attrs);
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

    field_reads.unzip()
}

/// Retuns field names and code to read those fields
pub(crate) fn read_named_fields(
    fields: &FieldsNamed,
) -> (Vec<Ident>, Vec<TokenStream>) {
    let field_reads = fields.named.iter().map(|field| {
        let name = field.ident.as_ref().unwrap().clone();

        let config = attrs::parse_field_attributes(&field.attrs);
        let FieldConfig { skip, width, align: _ } = config;

        let buf_size = skip + width;

        // TODO: we shouldn't need a String here at all
        let read = quote! {
            let mut s: [u8; #buf_size] = [0; #buf_size];
            buf.read_exact(&mut s).map_err(|e| fixed::error::Error::from(e))?;
            let #name = std::str::from_utf8(&s)
                .map_err(|e| fixed::error::Error::from_utf8_error(&s, e))?
                .parse_with(#config)
                .map_err(|e| fixed::error::Error::from(e))?;
        };

        (name, read)
    });

    field_reads.unzip()
}
