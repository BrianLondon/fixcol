//! Utilities for parsing field attributres
use proc_macro::{TokenStream, TokenTree};
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

struct FieldParam {
    key: String,
    value: String,
}

impl FieldParam {
    fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

// fn get_string_from_token(tt: TokenTree) -> String {
//     match tt {
//         TokenTree::Ident(ident) => ident.to_string(),
//         TokenTree::Literal(_) => todo!(),
//         TokenTree::Punct(p) => {
//             panic!("Expected identifier or literal {}", 
//                 p.span().source_text().unwrap_or("???".to_string()));
//         },
//         TokenTree::Group(g) => {
//             panic!("Expected identifier or literal {}", 
//                 g.span().source_text().unwrap_or("???".to_string()));
//         },
// }

// String holds the key of the current param we're parsing
// #[derive(PartialEq, Eq, Copy)]
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

fn parse_next_token(state: ExpectedTokenState, tt: TokenTree) -> 
        (ExpectedTokenState, Option<FieldParam>) 
{
    match (state, tt) {
        (ExpectedTokenState::Key, TokenTree::Ident(ident)) => {
            (ExpectedTokenState::Equals(ident.to_string()), None)
        },
        (ExpectedTokenState::Key, t) => {
            panic!("Expected identifier {}", 
                t.span().source_text().unwrap_or("???".to_string()));
        },
        (ExpectedTokenState::Equals(key), TokenTree::Punct(p)) if p.as_char() == '=' => {
            (ExpectedTokenState::Value(key), None)
        },        
        (ExpectedTokenState::Equals(_), t) => {
            panic!("Expected assignmen ('=' character) {}", 
                t.span().source_text().unwrap_or("???".to_string()));
        },
        (ExpectedTokenState::Value(key), TokenTree::Ident(ident)) => {
            let value = ident.to_string();
            (ExpectedTokenState::Separator, Some(FieldParam::new(key.to_string(), value)))
        },
        (ExpectedTokenState::Value(key), TokenTree::Literal(literal)) => {
            let value = literal.to_string();
            (ExpectedTokenState::Separator, Some(FieldParam::new(key, value)))
        },
        (ExpectedTokenState::Value(key), t) => {
            panic!("Expected identifier or literal {}", 
                t.span().source_text().unwrap_or("???".to_string()));
        },
        (ExpectedTokenState::Separator, TokenTree::Punct(p)) if p.as_char() == ',' => {
            (ExpectedTokenState::Key, None)
        },
        (ExpectedTokenState::Separator, t) => {
            panic!("Expected separator (',' character) or end of sequence {:?}",
                t.span());
        },
    }
}

pub fn get_config_params(tokens: TokenStream) {// -> Iterator<FieldParam> {
    let mut state = ExpectedTokenState::Key;
    let field_params = tokens.into_iter().scan(state, |s, t| {
        let (new_state, out) = parse_next_token(s.duplicate(), t);
        *s = new_state;
        out
    });

    
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
    use super::*;

    #[test]
    fn fail() {
        assert!(false);
    }
}
