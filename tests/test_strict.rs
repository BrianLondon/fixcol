extern crate fixcol;

use fixcol::ReadFixed;

#[derive(PartialEq, Eq, Debug, ReadFixed)]
#[fixcol(strict = true)]
struct PointS {
    #[fixcol(width = 3)]
    x: u8,
    #[fixcol(skip = 1, width = 3)]
    y: u8,
}

impl PointS {
    fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

#[derive(PartialEq, Eq, Debug, ReadFixed)]
#[fixcol(strict = false)]
struct PointL {
    #[fixcol(width = 3)]
    x: u8,
    #[fixcol(skip = 1, width = 3)]
    y: u8,
}

impl PointL {
    fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

// strict mode should require unread columns to contain only whitespace
///////////////////////////////////////////////////////////////////////

#[test]
fn whitespace_well_formed_lax() {
    let point = PointL::read_fixed_str("123 201").unwrap();
    assert_eq!(point, PointL::new(123, 201));
}

#[test]
fn whitespace_malformed_lax() {
    let point = PointL::read_fixed_str("1234201").unwrap();
    assert_eq!(point, PointL::new(123, 201));
}

#[test]
fn whitespace_well_formed_strict() {
    let point = PointS::read_fixed_str("123 201").unwrap();
    assert_eq!(point, PointS::new(123, 201));
}

#[test]
fn whitespace_malformed_strict() {
    let err = PointS::read_fixed_str("1234201").unwrap_err();
    assert_eq!(
        err.to_string(),
        "Error handling data from \"4201\": Found non-whitespace \
        character between data fields (strict)\n",
    );

    let err = PointS::read_fixed_str("1  42  ").unwrap_err();
    assert_eq!(
        err.to_string(),
        "Error handling data from \"42  \": Found non-whitespace \
        character between data fields (strict)\n",
    );
}

// strict mode should require last field of line to be full length when reading
///////////////////////////////////////////////////////////////////////////////

#[test]
fn short_line_lax() {
    println!("foo");
    let point = PointL::read_fixed_str("7   21").unwrap();
    assert_eq!(point, PointL::new(7, 21));
}

#[test]
fn short_line_strict() {
    let err = PointS::read_fixed_str("7   21").unwrap_err();
    // TODO: need better error messaging for this
    assert_eq!(err.to_string(), "failed to fill whole buffer");
}

#[test]
fn full_line_lax() {
    let point = PointL::read_fixed_str("7   21 ").unwrap();
    assert_eq!(point, PointL::new(7, 21));
}

#[test]
fn full_line_strict() {
    let point = PointS::read_fixed_str("7   21 ").unwrap();
    assert_eq!(point, PointS::new(7, 21));
}
