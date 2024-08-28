//! Utilities for parsing field attributres
use std::fmt::Display;
use std::str::FromStr;

use proc_macro2::{Literal, Span, TokenStream, TokenTree};
use quote::quote;
use syn::{spanned::Spanned, Attribute, Ident, Meta, Path};

use crate::error::MacroError;

const FIXED_ATTR_KEY: &'static str = "fixcol";
const STRICT_DEFAULT: bool = true;

// Extracts the ident name from a path
fn ident_from_path(path: &Path) -> String {
    path.segments
        .first()
        .map(|seg| seg.ident.to_string())
        .unwrap_or("".to_string())
}

// TODO: Update this method name
/// Indicates whether the attribute is used by Fixed
fn is_fixed_attr(attr: &Attribute) -> bool {
    let ident = match &attr.meta {
        Meta::Path(path) => ident_from_path(path),
        Meta::NameValue(named_value) => ident_from_path(&named_value.path),
        Meta::List(meta_list) => ident_from_path(&meta_list.path),
    };

    ident == FIXED_ATTR_KEY
}

// TODO: Update this function name
pub(crate) fn fixed_attrs(attrs: &Vec<Attribute>) -> Vec<&Attribute> {
    attrs.iter().filter(|a| is_fixed_attr(a)).collect()
}

/// Wraps either a literal or an identifier
#[derive(Debug)]
enum ValueToken {
    Ident(Ident),
    Literal(Literal),
}

impl ValueToken {
    fn span(&self) -> Span {
        match self {
            ValueToken::Ident(ident) => ident.span(),
            ValueToken::Literal(literal) => literal.span(),
        }
    }
}

impl Display for ValueToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueToken::Ident(ident) => ident.fmt(f),
            ValueToken::Literal(literal) => literal.fmt(f),
        }
    }
}

impl From<Ident> for ValueToken {
    fn from(value: Ident) -> Self {
        Self::Ident(value)
    }
}

impl From<Literal> for ValueToken {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

/// Wraps either a VariantConfig or a StructConfig to cascade to the field config
pub(crate) enum OuterConfig {
    Variant(VariantConfig),
    Struct(StructConfig),
}

impl OuterConfig {
    pub fn strict(&self) -> bool {
        match self {
            OuterConfig::Variant(vc) => vc.strict,
            OuterConfig::Struct(sc) => sc.strict,
        }
    }
}

impl From<VariantConfig> for OuterConfig {
    fn from(value: VariantConfig) -> Self {
        Self::Variant(value)
    }
}

impl From<StructConfig> for OuterConfig {
    fn from(value: StructConfig) -> Self {
        Self::Struct(value)
    }
}

/// Holds a parsed parameter from an attribute
///
/// The source for a `FieldParam`` started as "key = value" in an attribute
/// like `#[fixcol(key = value)]`. The field param itself holds the raw tokens
/// but the raw values can be extracted.
#[derive(Debug)]
struct FieldParam {
    key: Ident,
    value: ValueToken,
}

fn strip_quotes(s: &str) -> String {
    s.trim_end_matches('\"')
        .trim_start_matches('\"')
        .to_string()
}

impl FieldParam {
    fn new(key: Ident, value: ValueToken) -> Self {
        Self { key, value }
    }

    #[cfg(test)]
    fn test(key: &str, value: &str) -> Self {
        use quote::format_ident;

        Self {
            key: format_ident!("{}", key),
            value: ValueToken::Literal(Literal::from_str(value).unwrap()),
        }
    }

    fn key_span(&self) -> Span {
        self.key.span()
    }

    fn value_span(&self) -> Span {
        self.value.span()
    }

    fn key(&self) -> String {
        self.key.to_string()
    }

    fn value(&self) -> String {
        strip_quotes(self.value.to_string().as_str())
    }
}

impl PartialEq for FieldParam {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key() && self.value() == other.value()
    }
}

impl Eq for FieldParam {}

// Ident holds the key of the current param we're parsing
#[derive(PartialEq, Eq, Debug)]
enum ExpectedTokenState {
    Key,
    Equals(Ident),
    Value(Ident),
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
) -> Result<(ExpectedTokenState, Option<FieldParam>), MacroError> {
    match (state, tt) {
        (ExpectedTokenState::Key, TokenTree::Ident(ident)) => {
            Ok((ExpectedTokenState::Equals(ident), None))
        }
        (ExpectedTokenState::Key, t) => Err(MacroError::new("Expected identifier.", t.span())),
        (ExpectedTokenState::Equals(key), TokenTree::Punct(p)) if p.as_char() == '=' => {
            Ok((ExpectedTokenState::Value(key), None))
        }
        (ExpectedTokenState::Equals(_), t) => Err(MacroError::new(
            "Expected assignment ('=' character).",
            t.span(),
        )),
        (ExpectedTokenState::Value(key), TokenTree::Ident(ident)) => Ok((
            ExpectedTokenState::Separator,
            Some(FieldParam::new(key, ident.into())),
        )),
        (ExpectedTokenState::Value(key), TokenTree::Literal(literal)) => Ok((
            ExpectedTokenState::Separator,
            Some(FieldParam::new(key, literal.into())),
        )),
        (ExpectedTokenState::Value(_), t) => {
            Err(MacroError::new("Expected identifier or literal.", t.span()))
        }
        (ExpectedTokenState::Separator, TokenTree::Punct(p)) if p.as_char() == ',' => {
            Ok((ExpectedTokenState::Key, None))
        }
        (ExpectedTokenState::Separator, t) => Err(MacroError::new(
            "Expected separator (',' character) or end of sequence.",
            t.span(),
        )),
    }
}

fn parse_attributes(attrs: &Vec<Attribute>) -> Result<Vec<FieldParam>, MacroError> {
    let params: Vec<Result<Vec<FieldParam>, MacroError>> = attrs
        .iter()
        .filter(|a| is_fixed_attr(*a))
        .map(|a| -> Result<Vec<FieldParam>, MacroError> {
            match &a.meta {
                Meta::Path(_) => Err(MacroError::new(
                    "Could not read config from path style attribute. \
                        \n\nExpected parameters like #[fixcol(width = 4)]",
                    a.meta.span(),
                )),
                Meta::List(m) => get_config_params(m.tokens.clone()),
                Meta::NameValue(nv) => Err(MacroError::new(
                    "Could not read config from name/value style attribute. \
                        \n\nExpected parameters like #[fixcol(width = 4)]",
                    nv.value.span(),
                )),
            }
        })
        .collect();

    let params: Result<Vec<Vec<FieldParam>>, MacroError> = params.into_iter().collect();
    Ok(params?.into_iter().flatten().collect())
}

fn get_config_params(tokens: TokenStream) -> Result<Vec<FieldParam>, MacroError> {
    let mut any_tokens = false;
    let mut state = ExpectedTokenState::Key;
    let mut field_params: Vec<FieldParam> = Vec::new();

    let mut last_span = tokens.span();

    for token in tokens.into_iter() {
        any_tokens = true;
        last_span = token.span();
        let (new_state, out) = parse_next_token(state, token)?;
        state = new_state;
        if let Some(param) = out {
            field_params.push(param);
        }
    }

    if state != ExpectedTokenState::Separator && any_tokens {
        Err(MacroError::new(
            format!("Expected {} found end of input.", state).as_str(),
            last_span,
        ))
    } else {
        Ok(field_params)
    }
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
    pub(crate) strict: bool,
}

// This allows us to directly convert a FieldConfig (from the macro code)
// into a FieldDescription literal in the generated code
impl quote::ToTokens for FieldConfig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let FieldConfig { skip, width, align, strict } = &self;

        let alignment = match &align {
            Align::Left => quote! { fixcol::Alignment::Left },
            Align::Right => quote! { fixcol::Alignment::Right },
            Align::Full => quote! { fixcol::Alignment::Full },
        };

        tokens.extend(quote! {
            &fixcol::FieldDescription {
                skip: #skip,
                len: #width,
                alignment: #alignment,
                strict: #strict,
            }
        });
    }
}

struct FieldConfigBuilder {
    width: Option<usize>,
    skip: Option<usize>,
    align: Option<Align>,
    strict: Option<bool>,
}

impl FieldConfigBuilder {
    fn new() -> Self {
        Self { width: None, skip: None, align: None, strict: None }
    }
}

fn check_none<T>(key: &str, span: Span, opt: Option<T>) -> Result<(), MacroError> {
    match opt {
        Some(_) => Err(MacroError::new(
            format!("Duplicate values for {}", key).as_str(),
            span,
        )),
        None => Ok(()),
    }
}

pub(crate) fn parse_field_attributes(
    span: &Span,
    attrs: &Vec<Attribute>,
    parent: &OuterConfig,
) -> Result<FieldConfig, MacroError> {
    let params = parse_attributes(attrs)?;
    let mut conf = FieldConfigBuilder::new();

    for param in params {
        match param.key().as_str() {
            "skip" => {
                let err = "Expected numeric value for skip.";
                let val: usize = param
                    .value()
                    .to_string()
                    .parse()
                    .map_err(|_| MacroError::new(err, param.value_span()))?;
                let old = conf.skip.replace(val);
                check_none("skip", param.key_span(), old)?;
            }
            "width" => {
                let err = "Expected numeric value for width.";
                let val: usize = param
                    .value()
                    .to_string()
                    .parse()
                    .map_err(|_| MacroError::new(err, param.value_span()))?;
                let old = conf.width.replace(val);
                check_none("width", param.key_span(), old)?;
            }
            "align" => {
                let err = "Expected values for align are \"left\", \"right\", or \"full\".";
                let val: Align = param
                    .value()
                    .to_string()
                    .parse()
                    .map_err(|_| MacroError::new(err, param.value_span()))?;
                let old = conf.align.replace(val);
                check_none("align", param.key_span(), old)?;
            }
            "strict" => {
                let err = "Expected boolean value for parameter strict.";
                let val: bool = param
                    .value()
                    .to_string()
                    .parse()
                    .map_err(|_| MacroError::new(err, param.value_span()))?;
                let old = conf.strict.replace(val);
                check_none("strict", param.key_span(), old)?;

            }
            key => {
                return Err(MacroError::new(
                    format!("Unrecognized parameter \"{}\".", key).as_str(),
                    param.key_span(),
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
                strict: conf.strict.unwrap_or(parent.strict()),
            };

            Ok(fc)
        }
        None => Err(MacroError::new(
            "Width must be specified for all fields.",
            *span,
        )),
    }
}

// TODO: confirm these need to be public
pub(crate) struct StructConfigBuilder { 
    strict: Option<bool>,
}

impl StructConfigBuilder {
    pub fn new() -> Self {
        Self { strict: None }
    }
}

pub(crate) struct StructConfig {
    strict: bool,
}

pub(crate) fn parse_struct_attributes(
    attrs: &Vec<Attribute>,
) -> Result<StructConfig, MacroError> {
    let params = parse_attributes(attrs)?;
    let mut conf = StructConfigBuilder::new();

    for param in params {
        match param.key().as_str() {
            "strict" => {
                let err = "Expected numeric value for key_width.";
                let val: bool = param
                    .value()
                    .to_string()
                    .parse()
                    .map_err(|_| MacroError::new(err, param.value_span()))?;
                let old = conf.strict.replace(val);
                check_none("strict", param.key_span(), old)?;
            }
            key => {
                return Err(MacroError::new(
                    format!("Unrecognized parameter \"{}\".", key).as_str(),
                    param.key_span(),
                ));
            }
        }
    }

    let sc = StructConfig {
        strict: conf.strict.unwrap_or(STRICT_DEFAULT),
    };

    Ok(sc)
}

pub(crate) struct EnumConfigBuilder {
    ignore_others: Option<bool>,
    key_width: Option<usize>,
    strict: Option<bool>,
}

impl EnumConfigBuilder {
    pub fn new() -> Self {
        Self { ignore_others: None, key_width: None, strict: None }
    }
}

pub(crate) struct EnumConfig {
    pub _ignore_others: bool, // TODO: implement
    pub key_width: usize,
    pub strict: bool,
}

pub(crate) fn parse_enum_attributes(
    name: &Ident,
    attrs: &Vec<Attribute>,
) -> Result<EnumConfig, MacroError> {
    let params = parse_attributes(attrs)?;
    let mut conf = EnumConfigBuilder::new();

    for param in params {
        match param.key().as_str() {
            "ignore_others" => {
                let err = "Expected true or false for ignore_others.";
                let val: bool = param
                    .value()
                    .to_string()
                    .parse()
                    .map_err(|_| MacroError::new(err, param.value_span()))?;
                let old = conf.ignore_others.replace(val);
                check_none("ignore_others", param.key_span(), old)?;
            }
            "key_width" => {
                let err = "Expected numeric value for key_width.";
                let val: usize = param
                    .value()
                    .to_string()
                    .parse()
                    .map_err(|_| MacroError::new(err, param.value_span()))?;
                let old = conf.key_width.replace(val);
                check_none("key_width", param.key_span(), old)?;
            }
            "strict" => {
                let err = "Expected numeric value for key_width.";
                let val: bool = param
                    .value()
                    .to_string()
                    .parse()
                    .map_err(|_| MacroError::new(err, param.value_span()))?;
                let old = conf.strict.replace(val);
                check_none("strict", param.key_span(), old)?;
            }
            key => {
                return Err(MacroError::new(
                    format!("Unrecognized parameter \"{}\".", key).as_str(),
                    param.key_span(),
                ));
            }
        }
    }

    let key_width = conf.key_width.ok_or(MacroError::new(
        "The parameter 'key' must be provided for all enum variants.\n\n \
        Try adding #[fixcol(key_width = 10)] to this enum replacing \"10\" with \
        the width of your key.",
        name.span(),
    ))?;

    let ec = EnumConfig {
        _ignore_others: conf.ignore_others.unwrap_or(false),
        strict: conf.strict.unwrap_or(STRICT_DEFAULT),
        key_width,
    };

    Ok(ec)
}

pub(crate) struct VariantConfigBuilder {
    key: Option<String>,
    embed: Option<bool>,
    strict: Option<bool>,
}

impl VariantConfigBuilder {
    pub fn new() -> Self {
        Self { key: None, embed: None, strict: None }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VariantConfig {
    pub key: String,
    pub embed: bool,
    pub strict: bool,
}

pub(crate) fn parse_variant_attributes(
    name: &Ident,
    attrs: &Vec<Attribute>,
    parent: &EnumConfig,
) -> Result<VariantConfig, MacroError> {
    let params = parse_attributes(attrs)?;
    let mut conf = VariantConfigBuilder::new();

    for param in params {
        match param.key().as_str() {
            "key" => {
                let old = conf.key.replace(param.value());
                check_none("key", param.key_span(), old)?;
            }
            "embed" => {
                let err = "Expected true or false for embed.";
                let val: bool = param
                    .value()
                    .to_string()
                    .parse()
                    .map_err(|_| MacroError::new(err, param.value_span()))?;
                let old = conf.embed.replace(val);
                check_none("embed", param.key_span(), old)?;
            }
            "strict" => {
                let err = "Expected numeric value for key_width.";
                let val: bool = param
                    .value()
                    .to_string()
                    .parse()
                    .map_err(|_| MacroError::new(err, param.value_span()))?;
                let old = conf.strict.replace(val);
                check_none("strict", param.key_span(), old)?;
            }
            key => {
                return Err(MacroError::new(
                    format!("Unrecognized parameter \"{}\".", key).as_str(),
                    param.key_span(),
                ));
            }
        }
    }

    let key = conf.key.ok_or(MacroError::new(
        "The parameter key must be provided for all enum variants.\n\n \
        Try adding #[fixcol(key = \"<my key>\")] to this variant.",
        name.span(),
    ))?;

    let vc = VariantConfig {
        key: key,
        embed: conf.embed.unwrap_or(false),
        strict: conf.strict.unwrap_or(parent.strict),
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
        let actual = strip_quotes("\"foo\"");
        let expected = String::from("foo");

        assert_eq!(actual, expected);
    }

    #[test]
    fn strip_quotes_ignore() {
        let actual = strip_quotes("1");
        let expected = String::from("1");

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_zero_field_params() {
        let code: MetaList = syn::parse_str("fixcol()").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens).unwrap();

        assert_eq!(params.len(), 0);
    }

    #[test]
    fn parse_one_field_param() {
        let expected = FieldParam::test("align", "\"right\"");

        let code: MetaList = syn::parse_str("fixcol(align=right)").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens).unwrap();

        assert_eq!(params.len(), 1);
        assert_eq!(*(params.get(0)).unwrap(), expected);
    }

    #[test]
    fn parse_two_field_params() {
        let expected = vec![
            FieldParam::test("width", "3"),
            FieldParam::test("align", "\"right\""),
        ];
        let code: MetaList = syn::parse_str("fixcol(width=3, align = right)").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens).unwrap();

        assert_eq!(params, expected);
    }

    #[test]
    fn parse_three_field_params() {
        let expected = vec![
            FieldParam::test("skip", "1"),
            FieldParam::test("width", "3"),
            FieldParam::test("align", "\"right\""),
        ];
        let code: MetaList = syn::parse_str("fixcol(skip=1,width=3, align = right)").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens).unwrap();

        assert_eq!(params, expected);
    }

    #[test]
    fn parse_with_quotes() {
        let expected = FieldParam::test("align", "\"right\"");
        let code: MetaList = syn::parse_str("fixcol(align=\"right\")").unwrap();
        let params: Vec<FieldParam> = get_config_params(code.tokens).unwrap();

        assert_eq!(params.len(), 1);
        assert_eq!(*(params.get(0)).unwrap(), expected);
    }

    #[test]
    #[should_panic(expected = "Expected assignment found end of input.")]
    fn parse_params_ident_only() {
        let code: MetaList = syn::parse_str("fixcol(width)").unwrap();
        let x: Vec<FieldParam> = get_config_params(code.tokens).unwrap();
        println!("{:?}", x)
    }

    #[test]
    #[should_panic(expected = "Expected value found end of input.")]
    fn parse_params_ident_equal_only() {
        let code: MetaList = syn::parse_str("fixcol(width=)").unwrap();
        let x: Vec<FieldParam> = get_config_params(code.tokens).unwrap();
        println!("{:?}", x)
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: MacroError { message: \
         \"Expected separator (',' character) or end of sequence.\", span: Span }"
    )]
    fn parse_params_missing_comma() {
        let code: MetaList = syn::parse_str("fixcol(width=3 align = right)").unwrap();
        let _: Vec<FieldParam> = get_config_params(code.tokens).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: MacroError { message: \
        \"Expected separator (',' character) or end of sequence.\", span: Span }"
    )]
    fn parse_params_wrong_separator() {
        let code: MetaList = syn::parse_str("fixcol(width=3; align = right)").unwrap();
        let _: Vec<FieldParam> = get_config_params(code.tokens).unwrap();
    }
}
