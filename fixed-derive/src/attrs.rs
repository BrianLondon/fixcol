//! Utilities for parsing field attributres
use std::{fmt::Display, str::FromStr};

use proc_macro2::{TokenStream, TokenTree};
use syn::{Attribute, Meta, Path};

const FIXED_ATTR_KEY: &'static str = "fixed";

// Extracts the ident name from a path
fn ident_from_path(path: &Path) -> String {
    path.segments
        .first()
        .map(|seg| seg.ident.to_string())
        .unwrap_or("".to_string())
}

/// Indicates whether the attribute is used by Fixed
fn is_fixed_attr(attr: &Attribute) -> bool {
    let ident = match &attr.meta {
        Meta::Path(path) => ident_from_path(path),
        Meta::NameValue(named_value) => ident_from_path(&named_value.path),
        Meta::List(meta_list) => ident_from_path(&meta_list.path),
    };

    ident == FIXED_ATTR_KEY
}

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

fn parse_next_token(
    state: ExpectedTokenState,
    tt: TokenTree,
) -> (ExpectedTokenState, Option<FieldParam>) {
    match (state, tt) {
        (ExpectedTokenState::Key, TokenTree::Ident(ident)) => {
            (ExpectedTokenState::Equals(ident.to_string()), None)
        }
        (ExpectedTokenState::Key, t) => {
            panic!("Expected identifier. Found {}.", t.to_string());
        }
        (ExpectedTokenState::Equals(key), TokenTree::Punct(p)) if p.as_char() == '=' => {
            (ExpectedTokenState::Value(key), None)
        }
        (ExpectedTokenState::Equals(_), t) => {
            panic!(
                "Expected assignmen ('=' character). Found {}.",
                t.to_string()
            );
        }
        (ExpectedTokenState::Value(key), TokenTree::Ident(ident)) => {
            let value = ident.to_string();
            (
                ExpectedTokenState::Separator,
                Some(FieldParam::new(key.to_string(), value)),
            )
        }
        (ExpectedTokenState::Value(key), TokenTree::Literal(literal)) => {
            let value = literal.to_string();
            (
                ExpectedTokenState::Separator,
                Some(FieldParam::new(key, value)),
            )
        }
        (ExpectedTokenState::Value(_), t) => {
            panic!("Expected identifier or literal. Found {}.", t.to_string());
        }
        (ExpectedTokenState::Separator, TokenTree::Punct(p)) if p.as_char() == ',' => {
            (ExpectedTokenState::Key, None)
        }
        (ExpectedTokenState::Separator, t) => {
            panic!(
                "Expected separator (',' character) or end of sequence. Found {:?}.",
                t.to_string()
            );
        }
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

pub enum Align {
    Left,
    Right,
    Full,
}

impl FromStr for Align {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" | "\"left\"" => Ok(Align::Left),
            "right" | "\"right\"" => Ok(Align::Right),
            "full" | "\"full\"" => Ok(Align::Full),
            other => Err(format!("Unknown alignment type {}", other)),
        }
    }
}

pub struct FieldConfig {
    pub skip: usize,
    pub width: usize,
    pub align: Align,
}

struct FieldConfigBuilder {
    width: Option<usize>,
    skip: Option<usize>,
    align: Option<Align>,
}

impl FieldConfigBuilder {
    fn new() -> Self {
        Self {
            width: None,
            skip: None,
            align: None,
        }
    }
}

pub fn parse_attributes(attrs: &Vec<Attribute>) -> FieldConfig {
    let params = attrs.iter().filter(|a| is_fixed_attr(*a)).flat_map(|a| {
        let tokens = match &a.meta {
            Meta::Path(_) => todo!(),
            Meta::List(m) => &m.tokens,
            Meta::NameValue(_) => todo!(),
        };
        get_config_params(tokens.clone())
    });

    let mut conf = FieldConfigBuilder::new();

    for param in params {
        match param.key.as_str() {
            "skip" => {
                if conf.skip.is_none() {
                    conf.skip = Some(param.value.parse().unwrap());
                } else {
                    panic!("Duplicate values for skip");
                }
            }
            "width" => {
                if conf.width.is_none() {
                    conf.width = Some(param.value.parse().unwrap());
                } else {
                    panic!("Duplicate values for width");
                }
            }
            "align" => {
                if conf.align.is_none() {
                    conf.align = Some(param.value.parse().unwrap());
                } else {
                    panic!("Duplicate values for align");
                }
            }
            key => panic!("Unrecognized parameter {}", key),
        }
    }

    let width = match conf.width {
        Some(w) => w,
        None => panic!("Width must be specified for all fields"),
    };

    FieldConfig {
        skip: conf.skip.unwrap_or(0),
        align: conf.align.unwrap_or(Align::Left),
        width: width,
    }
}

#[cfg(test)]
mod tests {
    // TODO: needs tests not just of parsing but all the way to the field config
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
    fn parse_with_quotes() {
        let expected = FieldParam {
            key: "align".to_owned(),
            value: "\"right\"".to_owned(),
        };
        let code: MetaList = syn::parse_str("fixed(align=\"right\")").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens);

        assert_eq!(params.len(), 1);
        assert_eq!(*(params.get(0)).unwrap(), expected);
    }

    #[test]
    #[should_panic(expected = "Expected assignment found end of input.")]
    fn parse_params_ident_only() {
        let code: MetaList = syn::parse_str("fixed(width)").unwrap();
        let x: Vec<FieldParam> = get_config_params(code.tokens);
        println!("{:?}", x)
    }

    #[test]
    #[should_panic(expected = "Expected value found end of input.")]
    fn parse_params_ident_equal_only() {
        let code: MetaList = syn::parse_str("fixed(width=)").unwrap();
        let x: Vec<FieldParam> = get_config_params(code.tokens);
        println!("{:?}", x)
    }

    #[test]
    #[should_panic(
        expected = "Expected separator (',' character) or end of sequence. Found \"align\"."
    )]
    fn parse_params_missing_comma() {
        let code: MetaList = syn::parse_str("fixed(width=3 align = right)").unwrap();
        let _: Vec<FieldParam> = get_config_params(code.tokens);
    }

    #[test]
    #[should_panic(
        expected = "Expected separator (',' character) or end of sequence. Found \";\"."
    )]
    fn parse_params_wrong_separator() {
        let code: MetaList = syn::parse_str("fixed(width=3; align = right)").unwrap();
        let _: Vec<FieldParam> = get_config_params(code.tokens);
    }
}
