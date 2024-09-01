#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! A crate used for *fixed* width *column* serialization and deserialization
//!
//! Fixcol provides a derive based deserialization framework for parsing text files
//! with fixed column width serialization formats. While file formats using character
//! delimeters such as CSV or JSON are more common today, fixed column file formats
//! are more naturally human readable and many older data sets, especially public
//! domain data sets continue to use them.
//!
//! The library is built around the [`ReadFixed`] trait which is ordinarily
//! derived on a data type (`struct` or `enum`) that represents a row of the data
//! file. The `fixcol` attribute is used to define how fields map to the schema.
//!
//! For writing data files rudimentary serialization is provided by [`WriteFixed`]
//! and [`WriteFixedAll`] behind the `experimental-write` feature flag.
//!
//! ## Examples
//! ### Basic Example
//!
//! Consider the following data file:
//!
//! ```text
//! Tokyo       13515271   35.689  139.692
//! Delhi       16753235   28.610   77.230
//! Shanghai    24870895   31.229  121.475
//! São Paulo   12252023  -23.550  -46.333
//! Mexico City  9209944   19.433  -99.133
//! ```
//!
//! We can create a basic data structure corresponding to the records in the file
//! and then read the data file as shown.
//!
//! ```
//! use fixcol::ReadFixed;
//! # use std::fs::File;
//!
//! #[derive(ReadFixed)]
//! # #[derive(Debug, PartialEq)]
//! struct City {
//!     #[fixcol(width = 12)]
//!     name: String,
//!     #[fixcol(width = 8, align = "right")]
//!     population: u64,
//!     #[fixcol(skip = 1, width = 8, align = "right")]
//!     lat: f32,
//!     #[fixcol(skip = 1, width = 8, align = "right")]
//!     lon: f32,
//! }
//!
//! # // TODO: Unicode support :/ make São Paulo work
//! # fn f() {
//! let mut file = File::open("cities.txt");
//! # }
//! # let mut file = "Tokyo       13515271   35.689  139.692
//! # Delhi       16753235   28.610   77.230
//! # Shanghai    24870895   31.229  121.475
//! # Sao Paulo   12252023  -23.550  -46.333
//! # Mexico City  9209944   19.433  -99.133".as_bytes();
//! let cities: Vec<City> = City::read_fixed_all(file)
//!     .map(|res| match res {
//!         Ok(city) => city,
//!         Err(err) => {
//!             eprintln!("{}", err);
//!             std::process::exit(1);
//!         }
//!     })
//!     .collect();
//!
//! assert_eq!(
//!     cities,
//!     vec![
//!         City {
//!             name: "Tokyo".into(),
//!             population: 13515271,
//!             lat: 35.689,
//!             lon: 139.692
//!         },
//!         City {
//!             name: "Delhi".into(),
//!             population: 16753235,
//!             lat: 28.610,
//!             lon: 77.230
//!         },
//!         City {
//!             name: "Shanghai".into(),
//!             population: 24870895,
//!             lat: 31.229,
//!             lon: 121.475
//!         },
//!         City {
//!             name: "Sao Paulo".into(),
//!             population: 12252023,
//!             lat: -23.550,
//!             lon: -46.333
//!         },
//!         City {
//!             name: "Mexico City".into(),
//!             population: 9209944,
//!             lat: 19.433,
//!             lon: -99.133
//!         },
//!     ]
//! );
//! ```
//!
//! ### Multiple Record Types
//!
//! Many data files contain lines corresponding to multiple types of records.
//! Typically the record type is indicated by the first few columns of the line.
//! In Fixcol we call this the *key* of the record. Multiple record types can be
//! decoded using an `enum` with a key annotation.
//!
//! Consider a directed graph with named nodes defined in a data file like the
//! following.
//!
//! ```text
//! NODE 001 Item A
//! NODE 002 Item B
//! EDGE 001 002
//! ```
//!
//! This file can be parsed with an enum like the following.
//!
//! ```
//! use fixcol::ReadFixed;
//!
//! # #[derive(PartialEq, Debug)]
//! #[derive(ReadFixed)]
//! #[fixcol(key_width = 4)]
//! enum GraphItem {
//!     #[fixcol(key = "NODE")]
//!     Node {
//!         #[fixcol(skip = 1, width = 3)]
//!         id: u8,
//!         #[fixcol(skip = 1, width = 6)]
//!         name: String,
//!     },
//!     #[fixcol(key = "EDGE")]
//!     Edge {
//!         #[fixcol(skip = 1, width = 3)]
//!         from_id: u8,
//!         #[fixcol(skip = 1, width = 3)]
//!         to_id: u8,
//!     },
//! }
//! # let mut buf = "NODE 001 Item A
//! # NODE 002 Item B
//! # EDGE 001 002".as_bytes();
//! # let graph: Vec<GraphItem> = GraphItem::read_fixed_all(buf)
//! #     .map(|r| r.unwrap())
//! #     .collect();
//! # assert_eq!(graph, vec![
//! #     GraphItem::Node { id: 1, name: "Item A".to_string() },
//! #     GraphItem::Node { id: 2, name: "Item B".to_string() },
//! #     GraphItem::Edge { from_id: 1, to_id: 2 },
//! # ]);
//! ```
//!  
//! ### Embedded Variants
//!
//! Often instead of having fields defined directly on an `enum` variant it is
//! convenient to *embed* a struct within a single parameter named tuple
//! variant. Fixcol supports this pattern. To use it, derive [`ReadFixed`] on
//! the inner type and use the `embed` parameter on the variant.
//!
//! ```
//! # use fixcol::ReadFixed;
//! # #[derive(PartialEq, Debug)]
//! #[derive(ReadFixed)]
//! struct Node {
//!     #[fixcol(skip = 1, width = 3)]
//!     id: u8,
//!     #[fixcol(skip = 1, width = 6)]
//!     name: String,
//! }
//!
//! # #[derive(PartialEq, Debug)]
//! #[derive(ReadFixed)]
//! struct Edge {
//!     #[fixcol(skip = 1, width = 3)]
//!     from_id: u8,
//!     #[fixcol(skip = 1, width = 3)]
//!     to_id: u8,
//! }
//!
//! # #[derive(PartialEq, Debug)]
//! #[derive(ReadFixed)]
//! #[fixcol(key_width = 4)]
//! enum GraphItem {
//!     #[fixcol(key = "NODE", embed = true)]
//!     Node(Node),
//!     #[fixcol(key = "EDGE", embed = true)]
//!     Edge(Edge),
//! }
//! # let mut buf = "NODE 001 Item A
//! # NODE 002 Item B
//! # EDGE 001 002".as_bytes();
//! # let graph: Vec<GraphItem> = GraphItem::read_fixed_all(buf)
//! #     .map(|r| r.unwrap())
//! #     .collect();
//! # assert_eq!(graph, vec![
//! #     GraphItem::Node(Node { id: 1, name: "Item A".to_string() }),
//! #     GraphItem::Node(Node { id: 2, name: "Item B".to_string() }),
//! #     GraphItem::Edge(Edge { from_id: 1, to_id: 2 }),
//! # ]);
//! ```
//!
//! ## Strict Mode
//!
//! Strict mode may be toggled on or off setting the appropriate `fixcol` attribute
//! like `#[fixcol(strict = true)]`. When strict mode is disabled, Fixcol will
//! try it's best to recover encoding errors. When enabled, many more unexpected
//! conditions will be reported as errors.
//!
//! Strict mode is currently enabled by default, but **this may change** in a
//! future version.
//!
//! The `strict` parameter can be applied to a `struct` or `enum`, `enum` variant,
//! or field. The setting will cascade to other levels with the innermost explicit
//! application of the `strict` parameter controlling.
//!
//! #### Example
//!
//! ```
//! # use fixcol::ReadFixed;
//! #[derive(ReadFixed)]
//! #[fixcol(strict = false)] // Inner elements will not be parsed in strict mode
//! struct Point {
//!     #[fixcol(width = 3)]
//!     x: u8, // Strict mode not will be applied
//!     #[fixcol(width = 3, strict = true)]
//!     y: u8, // Strict mode will be applied
//!     #[fixcol(width = 3)]
//!     z: u8, // Strict mode not will be applied
//! }
//! ```
//!
//! #### Strict mode effects
//!
//! When a given field is parsed in strict mode the following conditions become
//! errors.
//! - The last field on a line is not whitespace padded to the defined length.
//! - Columns between defined data columns contain non-whitespace characters.
//! - Numeric column defined with `Full` alignment are not zero-padded to the
//!   full length.
//! - A `Left` aligned field beginning with whitespace.
//! - A `Right` aligned field ending with whitespace.
//!
//! Additional rules are applied while attempting to write a record. The following
//! are errors in strict mode.
//! - A `Full` aligned `String` field that is not the expected full length. That
//!   is, the supplied string must either be naturally the correct length or
//!   explicitly whitespace padded to be.
//! - Value supplied for any column that would overflow the allowed space.
//!
//! ## Schema Definition Parameters
//!
//! Fixcol defines serialization and deserialization schemas using `fixcol`
//! annotations that contain a list of one or more parameters in the form
//! `#[fixcol(param1 = value1, param2 = value2, ...)]`.
//!
//! #### Align
//!
//! Indicates the text alignment of the specified field.
//!
//! **Can be applied to**: Field
//!
//! **Allowed Values**: `"left"`, `"right"`, `"full"`
//!
//! | Value | Meaning |
//! |-------|---------|
//! | Left  | The value is left aligned and trailing whitespace can be ignored |
//! | Right | The caule is right aligned and leading whitespace can be ignored |
//! | Full  | The value is expected to occupy the full defined width. Leading and trailing whitespace are considered significant. |
//!
//! The values of the `align` parameter are mapped to an instance of [`Alignment`]
//! internally.
//!
//! **Default**: Left
//!
//! **Example**: `#[fixcol(width = 6, align = "right")]`
//!
//! #### Embed
//!
//! When decoding a single valued tuple-style enum variant, use the [`ReadFixed`]
//! implementation on the inner type.
//!
//! **Can be applied to**: Enum Variant
//!
//! **Allowed Values**: `true`, `false`
//!
//! **Default**: `false`
//!
//! **Example**: `#[fixcol(embed = true)]`
//!
//! #### Key
//!
//! When decoding multiple record types into an enum, indicates the key that
//! signifies this particular enum variant should be used to decode the line.
//!
//! To encode keys of different lengths, space pad the shorter keys so that all
//! declared keys are explicitly `key_width` characters.
//!
//! **Can be applied to**: Enum Variant
//!
//! **Allowed Values**: Strings of length `key_width`
//!
//! **Default**: Must be set **explicitly**.
//!
//! **Example**: `#[fixcol(key = "EDGE")]`
//!
//! #### Key Width
//!
//! When decoding multiple record types into an enum, indicates how many characters
//! at the begining of each line should be considered the *key* used to identify
//! which record type the line contains and therefore which enum variant should
//! be used to decode the line.
//!
//! **Can be applied to**: Enum
//!
//! **Allowed Values**: Positive integers
//!
//! **Default**: Must be set **explicitly**.
//!
//! **Example**: `#[fixcol(key_width = 4)]`
//!
//! #### Skip
//!
//! Indicates the number of columns (measured in bytes) that are expected to be
//! blank between the prior data field and the current data field.
//!
//! **Can be applied to**: Field
//!
//! **Allowed Values**: Non-negative integers
//!
//! **Default**: Zero
//!
//! **Example**: `#[fixcol(skip = 1, width = 12)]`
//!
//! #### Strict
//!
//! Indicates whether [strict mode](crate#strict-mode) should be enabled (See above).
//!
//! **Can be applied to**: Struct, Enum, Enum Varriant, Field
//!
//! **Allowed Values**: `true`, `false`
//!
//! **Default**: Cascades from outer context, outermost default `true`.
//!
//! **Example**: `#[fixcol(strict = true)]`
//!
//!
//! #### Width
//!
//! Indicates the number of columns (measured in bytes) used to encode the
//! target field.
//!
//! **Can be applied to**: Field
//!
//! **Allowed Values**: Positive integers
//!
//! **Default**: Must be set **explicitly**.
//!
//! **Example**: `#[fixcol(width = 12)]`

pub mod error;
mod fixcol;
mod format;
mod parse;

#[cfg(feature = "experimental-write")]
mod write;

extern crate fixcol_derive;

pub use fixcol::{Iter, ReadFixed};
#[cfg(feature = "experimental-write")]
pub use fixcol::{WriteFixed, WriteFixedAll};

pub use fixcol_derive::ReadFixed;
#[cfg(feature = "experimental-write")]
pub use fixcol_derive::WriteFixed;

pub use format::{Alignment, FieldDescription};
pub use parse::FixedDeserializer;
#[cfg(feature = "experimental-write")]
pub use write::FixedSerializer;

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use error::Error;

    use super::*;

    fn to_str(inp: Vec<u8>) -> String {
        use std::str;
        str::from_utf8(inp.as_slice()).unwrap().to_string()
    }

    #[test]
    fn test_helper() {
        let buf: [u8; 13] = [72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33];

        let mut v = Vec::new();
        let _ = v.write(&buf);

        assert_eq!(to_str(v), "Hello, World!");
    }

    #[test]
    #[cfg(feature = "experimental-write")]
    fn write_custom_basic() {
        struct Foo;

        impl WriteFixed for Foo {
            fn write_fixed<W: Write>(&self, buf: &mut W) -> Result<(), Error> {
                let _ = buf.write("Foo".as_bytes())?;
                Ok(())
            }
        }

        let foo = Foo;

        let mut v = Vec::new();
        let res = foo.write_fixed(&mut v);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "Foo");
    }

    // Name is left aligned, ten characters
    // value is right aligned three characters
    #[derive(Debug, PartialEq, Eq)]
    struct NumWord {
        name: String,
        value: u8,
    }

    #[cfg(feature = "experimental-write")]
    impl WriteFixed for NumWord {
        fn write_fixed<W: Write>(&self, buf: &mut W) -> Result<(), Error> {
            let _ = buf.write_fmt(format_args!("{:<10}{:>3}", self.name, self.value))?;
            Ok(())
        }
    }

    impl ReadFixed for NumWord {
        fn read_fixed<R: Read>(buf: &mut R) -> Result<Self, Error>
        where
            Self: Sized,
        {
            let mut s = String::new();
            let _ = buf.read_to_string(&mut s);

            let name = s[0..10].trim_end().to_string();
            let num = s[10..].to_string();
            let value = num.trim_start().parse::<u8>().unwrap();

            Ok(NumWord { name, value })
        }
    }

    #[cfg(feature = "experimental-write")]
    #[test]
    fn custom_struct_write() {
        let three = NumWord { name: "three".to_string(), value: 3 };

        let mut v = Vec::new();
        let res = three.write_fixed(&mut v);

        assert!(res.is_ok());
        assert_eq!(to_str(v), "three       3");
    }

    #[test]
    fn custom_struct_read() {
        let three = NumWord { name: "three".to_string(), value: 3 };

        let mut buf = "three       3".as_bytes();
        let decoded = NumWord::read_fixed(&mut buf).unwrap();

        assert_eq!(decoded, three);
    }
}
