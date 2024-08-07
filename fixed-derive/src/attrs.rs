//! Utilities for parsing field attributres
use std::{fmt::Display, str::FromStr};

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{Attribute, Ident, Meta, Path};

use crate::error::MacroError;

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

pub(crate) fn fixed_attrs(attrs: &Vec<Attribute>) -> Vec<&Attribute> {
    attrs.iter().filter(|a| is_fixed_attr(a)).collect()
}

#[derive(Debug)]
struct FieldParam {
    key: String,
    value: String,
    span: Option<Span>,
}

impl FieldParam {
    fn new(key: String, value: String, span: Span) -> Self {
        Self { key, value, span: Some(span) }
    }

    #[cfg(test)]
    fn spanless(key: &str, value: &str) -> Self {
        Self { 
            key: String::from(key),
            value: String::from(value),
            span: None 
        }
    }
}

impl PartialEq for FieldParam {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl Eq for FieldParam {}

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

fn strip_quotes(s: String) -> String {
    s.trim_end_matches('\"')
        .trim_start_matches('\"')
        .to_string()
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
                Some(FieldParam::new(key.to_string(), value, ident.span())),
            )
        }
        (ExpectedTokenState::Value(key), TokenTree::Literal(literal)) => {
            let value = strip_quotes(literal.to_string());
            (
                ExpectedTokenState::Separator,
                Some(FieldParam::new(key, value, literal.span())),
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

fn parse_attributes(attrs: &Vec<Attribute>) -> Vec<FieldParam> {
    attrs
        .iter()
        .filter(|a| is_fixed_attr(*a))
        .flat_map(|a| {
            let tokens = match &a.meta {
                Meta::Path(_) => unimplemented!("Could not parse Meta::Path -- unreachable?"),
                Meta::List(m) => &m.tokens,
                Meta::NameValue(_) => unimplemented!("Could not parse Meta::NameValue"),
            };

            get_config_params(tokens.clone())
        })
        .collect()
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

pub(crate) enum Align {
    Left,
    Right,
    Full,
}

impl FromStr for Align {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Align::Left),
            "right" => Ok(Align::Right),
            "full" => Ok(Align::Full),
            other => Err(format!("Unknown alignment type {}", other)),
        }
    }
}

pub(crate) struct FieldConfig {
    pub(crate) skip: usize,
    pub(crate) width: usize,
    pub(crate) align: Align,
}

struct FieldConfigBuilder {
    width: Option<usize>,
    skip: Option<usize>,
    align: Option<Align>,
}

impl FieldConfigBuilder {
    fn new() -> Self {
        Self { width: None, skip: None, align: None }
    }
}

pub(crate) fn parse_field_attributes(
    span: &Span,
    attrs: &Vec<Attribute>
) -> Result<FieldConfig, MacroError> {
    let params = parse_attributes(attrs);
    let mut conf = FieldConfigBuilder::new();

    for param in params {
        match param.key.as_str() {
            "skip" => {
                let err = format!("Expected numeric value for skip. Found {}", param.value);
                let old = conf.skip.replace(param.value.parse().expect(err.as_str()));
                if old.is_some() {
                    panic!("Duplicate values for skip");
                }
            }
            "width" => {
                let err = format!("Expected numeric value for width. Found {}", param.value);
                let old = conf.width.replace(param.value.parse().expect(err.as_str()));
                if old.is_some() {
                    panic!("Duplicate values for width");
                }
            }
            "align" => {
                let err = "Expected values for align are \"left\", \"right\", or \"full\".";
                let old = conf.align.replace(param.value.parse().expect(err));
                if old.is_some() {
                    panic!("Duplicate values for align");
                }
            }
            key => {
                return Err(MacroError::new(
                    format!("Unrecognized parameter \"{}\".", key).as_str(),
                    *span,
                ));
            }
        }
    }

    match conf.width {
        Some(width) => {
            let fc = FieldConfig {
                skip: conf.skip.unwrap_or(0),
                align: conf.align.unwrap_or(Align::Left),
                width: width,
            };
        
            Ok(fc)        
        },
        None => {
            Err(MacroError::new("Width must be specified for all fields.", *span))
        },
    }
}

pub(crate) struct EnumConfigBuilder {
    ignore_others: Option<bool>,
    key_width: Option<usize>,
}

impl EnumConfigBuilder {
    pub fn new() -> Self {
        Self { ignore_others: None, key_width: None }
    }
}

pub(crate) struct EnumConfig {
    pub ignore_others: bool, // TODO: implement
    pub key_width: usize,
}

pub(crate) fn parse_enum_attributes(
    name: &Ident,
    attrs: &Vec<Attribute>
) -> Result<EnumConfig, MacroError> {
    let params = parse_attributes(attrs);
    let mut conf = EnumConfigBuilder::new();

    for param in params {
        match param.key.as_str() {
            "ignore_others" => {
                let err = format!(
                    "Expected true or false for ignoer_others. Found {}",
                    param.value
                );
                let old = conf
                    .ignore_others
                    .replace(param.value.parse().expect(err.as_str()));
                if old.is_some() {
                    panic!("Duplicate values for ignore_others");
                }
            }
            "key_width" => {
                let err = format!(
                    "Expected numeric value for key_width. Found {}",
                    param.value
                );
                let old = conf
                    .key_width
                    .replace(param.value.parse().expect(err.as_str()));
                if old.is_some() {
                    panic!("Duplicate values for key_width");
                }
            }
            key => panic!(
                "Fixed encountered an unrecognized parameter \"{}\" \
                while parsing enum {}",
                key, name
            ),
        }
    }

   let ec = EnumConfig {
        ignore_others: conf.ignore_others.unwrap_or(false),
        key_width: conf
            .key_width
            .expect("The parameter key_width must be provided for enums."),
    };

    Ok(ec)
}

pub(crate) struct VariantConfigBuilder {
    key: Option<String>,
    embed: Option<bool>,
}

impl VariantConfigBuilder {
    pub fn new() -> Self {
        Self { key: None, embed: None }
    }
}

pub(crate) struct VariantConfig {
    pub key: String,
    pub embed: bool,
}

pub(crate) fn parse_variant_attributes(
    name: &Ident,
    attrs: &Vec<Attribute>
) -> Result<VariantConfig, MacroError> {
    let params = parse_attributes(attrs);
    let mut conf = VariantConfigBuilder::new();

    for param in params {
        match param.key.as_str() {
            "key" => {
                let old = conf.key.replace(param.value);
                if old.is_some() {
                    panic!("Duplicate values for key");
                }
            }
            "embed" => {
                let embed = match param.value.as_str() {
                    "true" => true,
                    "false" => false,
                    v => {
                        panic!("Expected boolean value for embed parameter. \
                            Found \"{}\"", v)
                    }
                };
                let old = conf.embed.replace(embed);
                if old.is_some() {
                    panic!("Duplicate values for embed");
                }
            }
            key => panic!(
                "Unrecognized parameter \"{}\" on enum variant {}",
                key, name
            ),
        }
    }

    let key = conf.key.ok_or(MacroError::new(
        "The parameter key must be provided for all enum variants",
        name.span(),
    ))?;

    let vc = VariantConfig {
        key: key,
        embed: conf.embed.unwrap_or(false),
    };

    Ok(vc)
}

#[cfg(test)]
mod tests {
    // TODO: needs tests not just of parsing but all the way to the field config
    use syn::{self, MetaList};

    use super::*;

    #[test]
    fn strip_quotes_strip() {
        let inp = String::from("\"foo\"");

        let actual = strip_quotes(inp);
        let expected = String::from("foo");

        assert_eq!(actual, expected);
    }

    #[test]
    fn strip_quotes_ignore() {
        let inp = String::from("1");

        let actual = strip_quotes(inp);
        let expected = String::from("1");

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_zero_field_params() {
        let code: MetaList = syn::parse_str("fixed()").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens);

        assert_eq!(params.len(), 0);
    }

    #[test]
    fn parse_one_field_param() {
        let expected = FieldParam::spanless("align", "right");

        let code: MetaList = syn::parse_str("fixed(align=right)").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens);

        assert_eq!(params.len(), 1);
        assert_eq!(*(params.get(0)).unwrap(), expected);
    }

    #[test]
    fn parse_two_field_params() {
        let expected = vec![
            FieldParam::spanless("width", "3"),
            FieldParam::spanless("align", "right"),
        ];
        let code: MetaList = syn::parse_str("fixed(width=3, align = right)").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens);

        assert_eq!(params, expected);
    }

    #[test]
    fn parse_three_field_params() {
        let expected = vec![
            FieldParam::spanless("skip", "1"),
            FieldParam::spanless("width", "3"),
            FieldParam::spanless("align", "right"),
        ];
        let code: MetaList = syn::parse_str("fixed(skip=1,width=3, align = right)").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens);

        assert_eq!(params, expected);
    }

    #[test]
    fn parse_with_quotes() {
        let expected = FieldParam::spanless("align", "right");
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
