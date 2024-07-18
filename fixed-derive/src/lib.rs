extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

use syn::{Data, DataStruct, DeriveInput, Fields};
use quote::quote;

// For now as a PoC we're just assuming ten characters per field
fn struct_write(fields: Fields) -> proc_macro2::TokenStream {
    let fields = match fields  {
        Fields::Named(named_fields) => named_fields,
        Fields::Unnamed(_) => todo!(),
        Fields::Unit => todo!(),
    };

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
        Data::Enum(_) => panic!("Deriving ReadFixed on enums is not supported"),
        Data::Union(_) => panic!("Deriving ReadFixed on unions is not supported"),
    };

    let gen = quote! {
        impl #impl_generics crate::WriteFixed for #name #ty_generics #where_clause {
            #function_impl
        }
    };

    println!("{}", gen);

    gen.into()
}
