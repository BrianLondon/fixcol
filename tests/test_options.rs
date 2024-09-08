use fixcol::ReadFixed;
#[cfg(feature = "experimental-write")]
use fixcol::WriteFixed;

#[cfg_attr(feature = "experimental-write", derive(WriteFixed))]
#[derive(Debug, PartialEq, ReadFixed)]
struct Thing1 {
    #[fixcol(width = 5)]
    name: String,
    #[fixcol(width = 5, align = "right")]
    #[rustfmt::skip]
    x: Option::<f32>,
    #[fixcol(width = 5, align = "right")]
    y: Option<f32>,
}

#[derive(Debug, PartialEq, ReadFixed)]
struct Thing2 {
    #[fixcol(width = 5)]
    name: String,
    #[fixcol(width = 5, align = "right")]
    x: Option<f32>,
    #[fixcol(width = 5, align = "right")]
    y: f32,
}

#[derive(Debug, PartialEq, ReadFixed)]
struct Thing3 {
    #[fixcol(width = 5)]
    name: Option<String>,
    #[fixcol(width = 5, align = "right")]
    x: f32,
    #[fixcol(width = 5, align = "right")]
    y: f32,
}

#[derive(Debug, PartialEq, ReadFixed)]
struct Thing4 {
    #[fixcol(width = 5)]
    name: String,
    #[fixcol(width = 5, align = "right")]
    x: f32,
    #[fixcol(width = 5, align = "right")]
    y: f32,
}

#[test]
fn parse_option() {
    let actual = Thing1::read_fixed_str("foo   3.14   42").unwrap();
    let expected = Thing1 {
        name: String::from("foo"),
        x: Some(3.14),
        y: Some(42.0),
    };
    assert_eq!(actual, expected);
}

#[test]
fn parse_none() {
    let actual = Thing1::read_fixed_str("foo   3.14     ").unwrap();
    let expected = Thing1 {
        name: String::from("foo"),
        x: Some(3.14),
        y: None,
    };
    assert_eq!(actual, expected);

    let actual = Thing1::read_fixed_str("foo          42").unwrap();
    let expected = Thing1 {
        name: String::from("foo"),
        x: None,
        y: Some(42.0),
    };
    assert_eq!(actual, expected);
}

#[test]
fn err_on_non_option_empty() {
    let actual = Thing2::read_fixed_str("foo          42").unwrap();
    let expected = Thing2 {
        name: String::from("foo"),
        x: None,
        y: 42.0,
    };
    assert_eq!(actual, expected);

    let actual = Thing2::read_fixed_str("foo   3.14     ");
    assert!(actual.is_err());
    assert_eq!(
        actual.unwrap_err().to_string(),
        "Error handling data from \"\": cannot parse float from empty string\n"
    );
}

#[test]
fn option_vs_empty_string() {
    let actual = Thing3::read_fixed_str("foo   3.14   42").unwrap();
    let expected = Thing3 {
        name: Some(String::from("foo")),
        x: 3.14,
        y: 42.0,
    };
    assert_eq!(actual, expected);

    let actual = Thing3::read_fixed_str("      3.14   42").unwrap();
    let expected = Thing3 { name: None, x: 3.14, y: 42.0 };
    assert_eq!(actual, expected);

    let actual = Thing4::read_fixed_str("foo   3.14   42").unwrap();
    let expected = Thing4 {
        name: String::from("foo"),
        x: 3.14,
        y: 42.0,
    };
    assert_eq!(actual, expected);

    let actual = Thing4::read_fixed_str("      3.14   42").unwrap();
    let expected = Thing4 { name: String::from(""), x: 3.14, y: 42.0 };
    assert_eq!(actual, expected);
}

#[test]
#[cfg(feature = "experimental-write")]
fn write_option_some() {
    let thing = Thing1 {
        name: String::from("foo"),
        x: Some(3.14),
        y: Some(42.0),
    };

    let mut v = Vec::new();
    let res = thing.write_fixed(&mut v);

    assert!(res.is_ok());

    let text = std::str::from_utf8(v.as_slice()).unwrap();
    assert_eq!(text, "foo   3.14   42");
}

#[test]
#[cfg(feature = "experimental-write")]
fn write_option_none() {
    let thing = Thing1 {
        name: String::from("foo"),
        x: None,
        y: Some(42.0),
    };

    let mut v = Vec::new();
    let res = thing.write_fixed(&mut v);

    assert!(res.is_ok());

    let text = std::str::from_utf8(v.as_slice()).unwrap();
    assert_eq!(text, "foo          42");
}
