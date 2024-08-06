use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};

pub(crate) type MacroResult = Result<TokenStream, MacroError>;

#[derive(Debug)]
pub(crate) struct MacroError {
    message: String,
    span: Span,
}

impl MacroError {
    pub(crate) fn new(message: &str, span: Span) -> Self {
        Self { message: String::from(message), span }
    }
}

impl ToTokens for MacroError {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let message = self.message.clone();
        let span = self.span.clone();

        tokens.extend(quote_spanned! {
            span => 
            compile_error!(#message);
        });
    }
}
