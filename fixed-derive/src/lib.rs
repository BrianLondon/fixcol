mod attrs;

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use attrs::FieldConfig;
use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed};

fn struct_read(fields: Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(named_fields) => struct_read_fields(named_fields),
        Fields::Unnamed(unnamed_fields) => tuple_struct_read_fields(unnamed_fields),
        Fields::Unit => panic!("Cannot deserialize type with no inner data"),
    }
}

impl quote::ToTokens for FieldConfig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let FieldConfig { skip, width, align } = &self;

        let alignment = match &align {
            attrs::Align::Left => quote! { fixed::Alignment::Left },
            attrs::Align::Right => quote! { fixed::Alignment::Right },
            attrs::Align::Full => quote! { fixed::Alignment::Full },
        };

        tokens.extend(quote! {
            &fixed::FieldDescription {
                skip: #skip,
                len: #width,
                alignment: #alignment,
            }
        });
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

#[proc_macro_derive(ReadFixed, attributes(fixed))]
pub fn read_fixed_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let function_impl = match ast.data {
        Data::Struct(DataStruct { fields, .. }) => struct_read(fields),
        Data::Enum(_) => panic!("Deriving ReadFixed on enums is not supported"),
        Data::Union(_) => panic!("Deriving ReadFixed on unions is not supported"),
    };

    let gen = quote! {
        impl #impl_generics fixed::ReadFixed for #name #ty_generics #where_clause {
            #function_impl
        }
    };

    gen.into()
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

fn struct_write(fields: Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(named_fields) => write_named_fields(named_fields),
        Fields::Unnamed(unnamed_fields) => write_unnamed_fields(unnamed_fields),
        Fields::Unit => 
            panic!("Unit structs not supported. Cannot serialize data type that hold no data"),
    }
}

#[proc_macro_derive(WriteFixed, attributes(fixed))]
pub fn write_fixed_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let function_impl = match ast.data {
        Data::Struct(DataStruct { fields, .. }) => struct_write(fields),
        Data::Enum(_) => panic!("Deriving WriteFixed on enums is not supported"),
        Data::Union(_) => panic!("Deriving WriteFixed on unions is not supported"),
    };

    let gen = quote! {
        impl #impl_generics fixed::WriteFixed for #name #ty_generics #where_clause {
            #function_impl
        }
    };

    gen.into()
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn macro_test() {
        assert!(true);
    }
}
