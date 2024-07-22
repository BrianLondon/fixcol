mod attrs;

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

use syn::{Data, DataStruct, DeriveInput, Fields};
use quote::quote;


// For now as a PoC we're just assuming ten characters per field
fn struct_read(fields: Fields) -> proc_macro2::TokenStream {
    let fields = match fields  {
        Fields::Named(named_fields) => named_fields,
        Fields::Unnamed(_) => todo!(),
        Fields::Unit => todo!(),
    };

    let field_reads = fields.named.iter().map(|field| {
        let name = field.ident.as_ref().unwrap().clone();

        // TODO: we shouldn't need a String here at all 
        quote!{
            let mut s: [u8; 10] = [0; 10];
            let _ = buf.read_exact(&mut s);
            let #name = std::str::from_utf8(&s).unwrap().parse_with(&fixed::FieldDescription {
                skip: 0,
                len: 10,
                alignment: fixed::Alignment::Left,
            }).unwrap();
        }
    });
    let mut read_steps = proc_macro2::TokenStream::new();
    read_steps.extend(field_reads.into_iter());

    let struct_init = fields.named.iter().map(|field| {
        let name = field.ident.as_ref().unwrap().clone();
        quote!{ 
            #name,
        }
    });
    let mut field_names = proc_macro2::TokenStream::new();
    field_names.extend(struct_init.into_iter());
    
    quote!{
        fn read_fixed<R: std::io::Read>(buf: &mut R) -> Result<Self, ()> {
            use fixed::FixedDeserializer;
            #read_steps

            Ok(Self {
                #field_names
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

    // println!("{}", gen);

    gen.into()
}

// For now as a PoC we're just assuming ten characters per field
fn struct_write(fields: Fields) -> proc_macro2::TokenStream {
    let fields = match fields  {
        Fields::Named(named_fields) => named_fields,
        Fields::Unnamed(_) => todo!(),
        Fields::Unit => todo!(),
    };



    fields.named.iter().for_each(|field| {
        println!("\n\n{}\n------------", field.ident.as_ref().unwrap());
        field.attrs.iter().for_each(|attr| {
            println!("{:?}", attr)
        })
    });

    let field_writes = fields.named.iter().map(|field| {
        let name = field.ident.as_ref().unwrap().clone();
        quote!{
            let _ = buf.write_fmt(format_args!("{:<10}", self.#name));
        }
    });

    let mut write_steps = proc_macro2::TokenStream::new();
    write_steps.extend(field_writes.into_iter());

    quote!{
        fn write_fixed(&self, buf: &mut dyn std::io::Write) -> Result<(), ()> {
            #write_steps
            Ok(())
        }
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

    // println!("{}", gen);

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
