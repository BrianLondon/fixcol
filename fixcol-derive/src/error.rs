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

    pub(crate) fn replace_span(&self, span: Span) -> Self {
        Self { message: self.message.clone(), span }
    }
}

impl ToTokens for MacroError {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let message = format!("{} error: {}", env!("CARGO_PKG_NAME"), self.message);
        let span = self.span.clone();

        tokens.extend(quote_spanned! {
            span =>
            compile_error!(#message);
        });
    }
}
