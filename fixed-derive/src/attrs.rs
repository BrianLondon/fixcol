//! Utilities for parsing field attributres
use std::fmt::Display;

use proc_macro2::{TokenStream, TokenTree};
use syn::{Attribute, Ident, Meta, Path};


const FIXED_ATTR_KEY: &'static str = "fixed";


// Extracts the ident name from a path
fn ident_from_path(path: Path) -> String {
    path.segments.first()
        .map(|seg| {seg.ident.to_string()})
        .unwrap_or("".to_string())
}

/// Indicates whether the attribute is used by Fixed
pub fn is_fixed_attr(attr: Attribute) -> bool {
    let ident = match attr.meta {
        Meta::Path(path) => ident_from_path(path),
        Meta::NameValue(named_value) => ident_from_path(named_value.path),
        Meta::List(meta_list) => ident_from_path(meta_list.path),
    };

    ident == FIXED_ATTR_KEY
}

// valid field parameters
//
// width, skip, align

// valid struct params
// ??
#[derive(PartialEq, Eq, Debug)]
struct FieldParam {
    key: String,
    value: String,
}

impl FieldParam {
    fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

// String holds the key of the current param we're parsing
#[derive(PartialEq, Eq, Debug)]
enum ExpectedTokenState {
    Key,
    Equals(String),
    Value(String),
    Separator,
}

impl ExpectedTokenState {
    fn duplicate(&self) -> Self {
        match self {
            ExpectedTokenState::Key => ExpectedTokenState::Key,
            ExpectedTokenState::Equals(s) => ExpectedTokenState::Equals(s.to_owned()),
            ExpectedTokenState::Value(s) => ExpectedTokenState::Value(s.to_owned()),
            ExpectedTokenState::Separator => ExpectedTokenState::Separator,
        }
    }
}

impl Display for ExpectedTokenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedTokenState::Key => f.write_str("identifier"),
            ExpectedTokenState::Equals(_) => f.write_str("assignment"),
            ExpectedTokenState::Value(_) => f.write_str("value"),
            ExpectedTokenState::Separator => f.write_str("separator"),
        }
    }
}

fn parse_next_token(state: ExpectedTokenState, tt: TokenTree) -> 
        (ExpectedTokenState, Option<FieldParam>) 
{
    println!("{:?} -> {:?}", state, tt);

    match (state, tt) {
        (ExpectedTokenState::Key, TokenTree::Ident(ident)) => {
            (ExpectedTokenState::Equals(ident.to_string()), None)
        },
        (ExpectedTokenState::Key, t) => {
            panic!("Expected identifier. Found {}.", 
                t.to_string());
        },
        (ExpectedTokenState::Equals(key), TokenTree::Punct(p)) if p.as_char() == '=' => {
            (ExpectedTokenState::Value(key), None)
        },        
        (ExpectedTokenState::Equals(_), t) => {
            panic!("Expected assignmen ('=' character). Found {}.", 
                t.to_string());
        },
        (ExpectedTokenState::Value(key), TokenTree::Ident(ident)) => {
            let value = ident.to_string();
            (ExpectedTokenState::Separator, Some(FieldParam::new(key.to_string(), value)))
        },
        (ExpectedTokenState::Value(key), TokenTree::Literal(literal)) => {
            let value = literal.to_string();
            (ExpectedTokenState::Separator, Some(FieldParam::new(key, value)))
        },
        (ExpectedTokenState::Value(_), t) => {
            panic!("Expected identifier or literal. Found {}.", 
                t.to_string());
        },
        (ExpectedTokenState::Separator, TokenTree::Punct(p)) if p.as_char() == ',' => {
            (ExpectedTokenState::Key, None)
        },
        (ExpectedTokenState::Separator, t) => {
            panic!("Expected separator (',' character) or end of sequence. Found {:?}.",
                t.to_string());
        },
    }
}

fn get_config_params(tokens: TokenStream) -> Vec<FieldParam> {
    let mut any_tokens = false;
    let mut state = ExpectedTokenState::Key;
    let mut field_params: Vec<FieldParam> = Vec::new();

    for token in tokens.into_iter() {
        any_tokens = true;
        let (new_state, out) = parse_next_token(state, token);
        state = new_state;
        if let Some(param) = out {
            field_params.push(param);
        }
    }

    if state != ExpectedTokenState::Separator && any_tokens {
        panic!("Expected {} found end of input.", state);
    }

    field_params
}


    /*

    #[fixed(width=12)]
    
    Attribute { 
        pound_token: Pound, 
        style: AttrStyle::Outer, 
        bracket_token: Bracket, 
        meta: Meta::List { 
            path: Path { 
                leading_colon: None, 
                segments: [
                    PathSegment { 
                        ident: Ident { 
                            ident: "fixed", 
                            span: #0 bytes(169..174) 
                        }, 
                        arguments: PathArguments::None 
                    }
                ] 
            }, 
            delimiter: MacroDelimiter::Paren(Paren),
            tokens: TokenStream [
                Ident { ident: "width", span: #0 bytes(175..180) },
                Punct { ch: '=', spacing: Alone, span: #0 bytes(180..181) }, 
                Literal { kind: Integer, symbol: "12", suffix: None, span: #0 bytes(181..183) }
            ] 
        }
    }

    /// The y coordinate
    Attribute { 
        pound_token: Pound, 
        style: AttrStyle::Outer, 
        bracket_token: Bracket, 
        meta: Meta::NameValue { 
            path: Path { 
                leading_colon: None, 
                segments: [
                    PathSegment {
                        ident: Ident { 
                            ident: "doc", 
                            span: #0 bytes(227..247) 
                        }, 
                        arguments: PathArguments::None 
                    }
                ] 
            }, 
            eq_token: Eq, 
            value: Expr::Lit {
                attrs: [], 
                lit: Lit::Str { token: " The y coordinate" } 
            } 
        } 
    }

    #[fixed(width=8, strict=true)]
    Attribute { 
        pound_token: Pound, 
        style: AttrStyle::Outer, 
        bracket_token: Bracket, 
        meta: Meta::List { 
            path: Path { 
                leading_colon: None, 
                segments: [
                    PathSegment {
                        ident: Ident { ident: "fixed", span: #0 bytes(254..259) }, 
                        arguments: PathArguments::None 
                    }
                ] 
            }, 
            delimiter: MacroDelimiter::Paren(Paren),
            tokens: TokenStream [
                Ident {ident: "width", span: #0 bytes(260..265) }, 
                Punct { ch: '=', spacing: Alone, span: #0 bytes(265..266) }, 
                Literal { kind: Integer, symbol: "8", suffix: None, span: #0 bytes(266..267) }, 
                Punct { ch: ',', spacing: Alone, span: #0 bytes(267..268) }, 
                Ident { ident: "strict", span: #0 bytes(269..275) }, 
                Punct { ch: '=', spacing: Alone, span: #0 bytes(275..276) }, 
                Ident { ident: "true", span: #0 bytes(276..280) }
            ] 
        } 
    }

    #[allow(non_camel_case_types)]

Attribute { pound_token: Pound, style: AttrStyle::Outer, bracket_token: Bracket, meta: Meta::List { path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "allow", span: #0 bytes(289..294) }, arguments: PathArguments::None }] }, delimiter: MacroDelimiter::Paren(Paren), tokens: TokenStream [Ident { ident: "non_camel_case_types", span: #0 bytes(295..315) }] } }
    
    */

#[cfg(test)]
mod tests {
    use syn::{self, MetaList};

    use super::*;

    #[test]
    fn parse_zero_field_params() {
        let code: MetaList = syn::parse_str("fixed()").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens);

        assert_eq!(params.len(), 0);
    }

    #[test]
    fn parse_one_field_param() {
        let expected = FieldParam { 
            key: "align".to_owned(), 
            value: "right".to_owned(),
        };
        let code: MetaList = syn::parse_str("fixed(align=right)").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens);

        assert_eq!(params.len(), 1);
        assert_eq!(*(params.get(0)).unwrap(), expected);
    }

    #[test]
    fn parse_two_field_params() {
        let expected = vec![
            FieldParam {
                key: "width".to_owned(),
                value: "3".to_owned(),
            },
            FieldParam { 
                key: "align".to_owned(), 
                value: "right".to_owned(),
            },
        ];
        let code: MetaList = syn::parse_str("fixed(width=3, align = right)").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens);

        assert_eq!(params, expected);
    }

    #[test]
    fn parse_three_field_params() {
        let expected = vec![
            FieldParam {
                key: "skip".to_owned(),
                value: "1".to_owned(),
            },
            FieldParam {
                key: "width".to_owned(),
                value: "3".to_owned(),
            },
            FieldParam { 
                key: "align".to_owned(), 
                value: "right".to_owned(),
            },
        ];
        let code: MetaList = syn::parse_str("fixed(skip=1,width=3, align = right)").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens);

        assert_eq!(params, expected);
    }

    #[test]
    #[should_panic(expected="Expected assignment found end of input.")]
    fn parse_params_ident_only() {
        let code: MetaList = syn::parse_str("fixed(width)").unwrap();
        let x: Vec<FieldParam> = get_config_params(code.tokens);
        println!("{:?}", x)
    }

    #[test]
    #[should_panic(expected="Expected value found end of input.")]
    fn parse_params_ident_equal_only() {
        let code: MetaList = syn::parse_str("fixed(width=)").unwrap();
        let x: Vec<FieldParam> = get_config_params(code.tokens);
        println!("{:?}", x)
    }

    #[test]
    #[should_panic(expected="Expected separator (',' character) or end of sequence. Found \"align\".")]
    fn parse_params_missing_comma() {
        let code: MetaList = syn::parse_str("fixed(width=3 align = right)").unwrap();
        let _: Vec<FieldParam> = get_config_params(code.tokens);
    }

    #[test]
    #[should_panic(expected="Expected separator (',' character) or end of sequence. Found \";\".")]
    fn parse_params_wrong_separator() {
        let code: MetaList = syn::parse_str("fixed(width=3; align = right)").unwrap();
        let _: Vec<FieldParam> = get_config_params(code.tokens);
    }
}
