use std::io::{BufRead, BufReader, Lines, Read};
#[cfg(any(feature = "experimental-write", doc))]
use std::io::Write;
use std::marker::PhantomData;

use crate::error::Error;

/// Trait for writing to fixed width (column based) serialization
///
/// The `fixcol` library provides limited writing functionality out of the box.
/// `WriteFixed` is the main entry point to that serialization functionality.
/// While one can always manually implement `WriteFixed`, it is normally derived
/// using the proc macro, which offers full string and integer support and
/// limited floating point formatting.
#[cfg(feature = "experimental-write")]
// #[cfg(any(feature = "experimental-write", doc))]
pub trait WriteFixed {
    /// Writes the object into the supplied buffer
    ///
    /// Provides logic for serializing an instance of the object in the specified
    /// fixed column format.
    ///
    /// # Example
    ///
    /// ```
    /// # use fixcol::WriteFixed;
    /// # use std::io;
    /// #[derive(WriteFixed)]
    /// struct Point {
    ///     #[fixcol(width=3)]
    ///     x: u8,
    ///     #[fixcol(width=3)]
    ///     y: u8,
    /// }
    ///
    /// let mut buffer = Vec::new();
    ///
    /// let point = Point { x: 12, y: 7 };
    /// let res = point.write_fixed(&mut buffer);
    ///
    /// assert_eq!(std::str::from_utf8(&buffer).unwrap(), "12 7  ");
    /// ```
    fn write_fixed<W: Write>(&self, buf: &mut W) -> Result<(), Error>;
}

/// Implements writing a data set in a fixed width column format
///
/// This trait exposes the [`write_fixed_all`] method that allows serialization
/// of a set of objects to a buffer in a newline delimited, fixed column data
/// format. There is a blanket implementation on all collections that
/// implement [`IntoIterator`] when the inner class implements [`WriteFixed`].
/// This trait should not need any custom implementation.
///
/// [`write_fixed_all`]: crate::WriteFixedAll::write_fixed_all
///
/// # Example
/// ```
/// use fixcol::WriteFixed;
/// #[derive(WriteFixed)]
/// struct Point {
///     #[fixcol(width=3)] x: u8,
///     #[fixcol(width=3)] y: u8,
/// }
/// // Point implements WriteFixed
///
/// use fixcol::WriteFixedAll;
/// let v: Vec<Point> = Vec::new();
/// // Therefore Vec<Point> implements WriteFixedAll
/// ```
#[cfg(feature = "experimental-write")]
pub trait WriteFixedAll {
    /// Writes a set of objects to the supplied buffer (newline delimited)
    ///
    /// # Example
    /// ```
    /// # use fixcol::WriteFixed;
    /// # use std::fs::File;
    /// # use std::io;
    /// #[derive(WriteFixed)]
    /// struct Point {
    ///     #[fixcol(width=3)] x: u8,
    ///     #[fixcol(width=3)] y: u8,
    /// }
    ///
    /// let v: Vec<Point> = vec![
    ///     // data here...
    /// #   Point { x: 0, y: 3},
    /// #   Point { x: 123, y: 42},
    /// #   Point { x: 42, y: 123},
    /// ];
    /// # fn f() {
    /// let mut file = File::open("my_file.txt").unwrap();
    /// # }
    /// # let mut file: Vec<u8> = Vec::new();
    ///
    /// use fixcol::WriteFixedAll;
    /// v.write_fixed_all(&mut file);
    /// # let s = std::str::from_utf8(file.as_slice()).unwrap();
    /// # assert_eq!(s, "0  3  \n12342 \n42 123\n");
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental-write")))]
    fn write_fixed_all<W: Write>(self, buf: &mut W) -> Result<(), Error>;
}

/// Blanket implementation of WriteFixedAll for collections of `impl WriteFixed`
///
/// See also: [`WriteFixed`]
#[cfg(feature = "experimental-write")]
impl<T: WriteFixed, Iter: IntoIterator<Item = T>> WriteFixedAll for Iter {
    fn write_fixed_all<W: Write>(self, buf: &mut W) -> Result<(), Error> {
        for item in self.into_iter() {
            item.write_fixed(buf)?;
            buf.write("\n".as_bytes())?;
        }

        Ok(())
    }
}

/// Iterator over the deserialized lines of a fixed column file
///
/// Implements [`Iterator`] for `T`. This struct is created by a call to
/// [`read_fixed_all`].
///
/// [`read_fixed_all`]: ReadFixed::read_fixed_all
pub struct Iter<T, R>
where
    T: ReadFixed,
    R: Read,
{
    // TODO: it might be more performant do operate at a slighly lower level
    // than mapping over ther BufReader lines iterator. If we did that, we'd use
    // fields that look something like the following:
    //
    // read_buf: BufReader<R>,
    // line_buf: String,
    failed: bool,
    line: usize,
    lines: Lines<BufReader<R>>,
    t: PhantomData<T>,
}

impl<T: ReadFixed, R: Read> Iter<T, R> {
    fn new(read: R) -> Self {
        Self {
            lines: BufReader::new(read).lines(),
            line: 0,
            failed: false,
            t: PhantomData,
        }
    }
}

impl<T: ReadFixed, R: Read> Iterator for Iter<T, R> {
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            None
        } else {
            self.line += 1;
            match self.lines.next() {
                None => None,
                Some(Err(e)) => {
                    self.failed = true;
                    Some(Err(Error::IoError(e)))
                }
                Some(Ok(s)) => {
                    // TODO: think about whether we want to allow it to return the
                    // errored line and keep going
                    match T::read_fixed_string(s) {
                        Err(Error::DataError(err)) => {
                            let err_with_line = err.with_line(self.line);
                            Some(Err(Error::DataError(err_with_line)))
                        }
                        other => Some(other),
                    }
                }
            }
        }
    }
}

/// Trait for reading from fixed width (column based) serializaiton
///
/// This trait is the main entry point to using `fixcol` for deserializing
/// column delimited data files. This trait is not normally implemented manually
/// but derived using the [`fixcol_derive`] crate. The deserialization behavior
/// of individual columns is defined using the `#[fixcol(...)]` annotation.
pub trait ReadFixed {
    /// Reads an instance of the object from the supplied buffer
    ///
    /// Provides logic for deserializing an instance of the type read from a
    /// supplied buffer.
    ///
    /// # Example
    /// ```
    /// use fixcol::ReadFixed;
    /// use std::fs::File;
    /// use std::io;
    /// 
    /// #[derive(ReadFixed)]
    /// struct Foo {
    ///     #[fixcol(width=3)]
    ///     foo: String,
    ///     #[fixcol(width=3)]
    ///     bar: String,
    /// }
    ///
    /// let mut buffer: &[u8] = "foobar".as_bytes();
    /// let my_foo: Foo = Foo::read_fixed(&mut buffer).unwrap();
    /// # assert_eq!(my_foo.foo, "foo".to_string());
    /// # assert_eq!(my_foo.bar, "bar".to_string());
    /// ```
    fn read_fixed<R>(buf: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
        R: Read;

    /// Consumes a buffer returning objects of type `Self`
    ///
    /// Lazily reads the entier content of `buf` returning an [`Iterator`]
    /// over deserialized objects.
    ///
    /// # Example
    /// ```
    /// # use fixcol::ReadFixed;
    /// # use std::fs::File;
    /// # use std::io;
    /// #[derive(ReadFixed)]
    /// struct MyType {
    ///     // ...
    /// }
    ///
    /// # fn f() {
    /// let mut file = File::open("my_file.txt").unwrap();
    /// for res in MyType::read_fixed_all(file) {
    ///     match res {
    ///         Ok(my_type) => // my_type is of type MyType ... do something with it here
    /// #       {},
    ///         Err(_) => // handle error
    /// #       {},
    ///     }
    /// }
    /// # }
    /// ```
    fn read_fixed_all<R>(buf: R) -> Iter<Self, R>
    where
        Self: Sized,
        R: Read,
    {
        Iter::new(buf)
    }

    /// Reads an instance of the object fom a `&str`
    ///
    /// Deserializes a single item of the type from a fixed width representation
    /// of the object stored in a `&str`.
    ///
    /// # Examples
    ///
    /// We can parse directly from `str` literals
    /// ```
    /// # use fixcol::ReadFixed;
    /// # use fixcol::FixedDeserializer;
    /// # use fixcol::FieldDescription;
    /// #[derive(ReadFixed)]
    /// struct Point {
    ///     #[fixcol(width=3, align="right")]
    ///     x: u8,
    ///     #[fixcol(width=3, align="right")]
    ///     y: u8,
    /// }
    ///
    /// let point = Point::read_fixed_str(" 42  7").unwrap();
    /// assert_eq!(point.x, 42);
    /// assert_eq!(point.y, 7)
    /// ```
    ///
    /// It can also be useful to pull directly from slices.
    /// ```
    /// # use fixcol::{FixedDeserializer, FieldDescription, ReadFixed};
    /// # #[derive(ReadFixed)]
    /// # struct Point {
    /// #     #[fixcol(width=3)]
    /// #     x: u8,
    /// #     #[fixcol(width=3)]
    /// #     y: u8,
    /// # }
    /// let s = ">>12361 <<";
    /// let point = Point::read_fixed_str(&s[2..8]).unwrap();
    ///
    /// assert_eq!(point.x, 123);
    /// assert_eq!(point.y, 61);
    /// ```
    fn read_fixed_str(s: &str) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut bytes = s.as_bytes();
        Self::read_fixed(&mut bytes)
    }

    /// Reads an instance of the object fom a [`String`]
    ///
    /// Deserializes a single item of the type from a fixed width representation
    /// of the object stored in a `String`.
    ///
    /// # Examples
    ///
    /// We can parse directly from `str` literals
    /// ```
    /// # use fixcol::ReadFixed;
    /// # use fixcol::FixedDeserializer;
    /// # use fixcol::FieldDescription;
    /// #[derive(ReadFixed)]
    /// struct Point {
    ///     #[fixcol(width=3, align="right")]
    ///     x: u8,
    ///     #[fixcol(width=3, align="right")]
    ///     y: u8,
    /// }
    ///
    /// let s = String::from(" 42  7");
    /// let point = Point::read_fixed_string(s).unwrap();
    /// assert_eq!(point.x, 42);
    /// assert_eq!(point.y, 7)
    /// ```
    fn read_fixed_string(s: String) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut bytes = s.as_bytes();
        Self::read_fixed(&mut bytes)
    }
}

#[cfg(test)]
mod tests {
    use fixcol_derive::ReadFixed;

    use super::*;
    use crate::error::Error;

    #[derive(Debug, PartialEq, Eq)]
    struct Foo {
        foo: String,
    }

    impl ReadFixed for Foo {
        fn read_fixed<R>(buf: &mut R) -> Result<Self, Error>
        where
            Self: Sized,
            R: Read,
        {
            let mut s: String = String::new();
            buf.read_to_string(&mut s)?;

            Ok(Self { foo: s })
        }
    }

    #[test]
    fn read_fixed_str() {
        let foo = Foo::read_fixed_str("bar");
        assert_eq!(foo.unwrap(), Foo { foo: "bar".to_string() });
    }

    #[test]
    fn read_fixed_string() {
        let s: String = "bar".to_string();
        let foo = Foo::read_fixed_string(s);
        assert_eq!(foo.unwrap(), Foo { foo: "bar".to_string() });
    }

    #[test]
    fn read_fixed_all() {
        let buf = "foo\nbar\nbaz\n";

        let expected = vec![
            Foo { foo: "foo".to_string() },
            Foo { foo: "bar".to_string() },
            Foo { foo: "baz".to_string() },
        ];

        let actual: Vec<Foo> = Foo::read_fixed_all(buf.as_bytes())
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_fixed_all_no_trailing() {
        let buf = "foo\nbar\nbaz";

        let expected = vec![
            Foo { foo: "foo".to_string() },
            Foo { foo: "bar".to_string() },
            Foo { foo: "baz".to_string() },
        ];

        let actual: Vec<Foo> = Foo::read_fixed_all(buf.as_bytes())
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(actual, expected);
    }

    // Derive tests (struct)
    ////////////////////////////////
    
    // Helper function only used in write tests
    #[cfg(feature = "experimental-write")]
    fn to_str(inp: Vec<u8>) -> String {
        use std::str;
        str::from_utf8(inp.as_slice()).unwrap().to_string()
    }

    use crate as fixcol;
    #[cfg(feature = "experimental-write")]
    use fixcol::WriteFixed;

    #[cfg_attr(feature = "experimental-write", derive(WriteFixed))]
    #[derive(ReadFixed, Eq, PartialEq, Debug)]
    struct MyStruct {
        #[fixcol(width = 10)]
        string: String,
        #[fixcol(width = 10, align = "right")]
        num: i64,
    }

    #[test]
    fn read_struct_derived() {
        let expected = MyStruct {
            string: "my string".to_string(),
            num: 981,
        };

        let raw = "my string        981";
        assert_eq!(raw.len(), 20);
        let mut buf = raw.as_bytes();
        let actual = MyStruct::read_fixed(&mut buf);

        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    #[cfg(feature = "experimental-write")]
    fn write_struct_derived() {
        let expected = "my string        981";
        assert_eq!(expected.len(), 20);

        let my_struct = MyStruct {
            string: "my string".to_string(),
            num: 981,
        };
        let mut buf: Vec<u8> = Vec::new();
        let res = my_struct.write_fixed(&mut buf);

        assert!(res.is_ok());
        assert_eq!(to_str(buf), expected);
    }

    // Derive tests (enum)
    #[cfg_attr(feature = "experimental-write", derive(WriteFixed))]
    #[derive(ReadFixed, Eq, PartialEq, Debug)]
    #[fixcol(key_width = 2)]
    enum MyEnum {
        #[fixcol(key = "st")]
        Struct {
            #[fixcol(width = 10)]
            string: String,
            #[fixcol(width = 10, align = "right")]
            num: i64,
        },
        #[fixcol(key = "tu")]
        Tuple(
            #[fixcol(width = 10)] String,
            #[fixcol(width = 10, align = "right")] i64,
        ),
        #[fixcol(key = "un")]
        Unit,
    }

    #[test]
    fn read_struct_enum_derived() {
        let expected = MyEnum::Struct {
            string: "my string".to_string(),
            num: 981,
        };

        let raw = "stmy string        981";
        assert_eq!(raw.len(), 22);
        let mut buf = raw.as_bytes();
        let actual = MyEnum::read_fixed(&mut buf);

        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    #[cfg(feature = "experimental-write")]
    fn write_struct_enum_derived() {
        let expected = "stmy string        981";
        assert_eq!(expected.len(), 22);

        let my_struct = MyEnum::Struct {
            string: "my string".to_string(),
            num: 981,
        };
        let mut buf: Vec<u8> = Vec::new();
        let res = my_struct.write_fixed(&mut buf);

        assert!(res.is_ok());
        assert_eq!(to_str(buf), expected);
    }

    #[test]
    fn read_tuple_enum_derived() {
        let expected = MyEnum::Tuple("my string".to_string(), 981);

        let raw = "tumy string        981";
        assert_eq!(raw.len(), 22);
        let mut buf = raw.as_bytes();
        let actual = MyEnum::read_fixed(&mut buf);

        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    #[cfg(feature = "experimental-write")]
    fn write_tuple_enum_derived() {
        let expected = "tumy string        981";
        assert_eq!(expected.len(), 22);

        let my_struct = MyEnum::Tuple("my string".to_string(), 981);
        let mut buf: Vec<u8> = Vec::new();
        let res = my_struct.write_fixed(&mut buf);

        assert!(res.is_ok());
        assert_eq!(to_str(buf), expected);
    }

    #[test]
    fn read_unit_enum_derived() {
        let expected = MyEnum::Unit;

        let raw = "unmy string        981";
        assert_eq!(raw.len(), 22);
        let mut buf = raw.as_bytes();
        let actual = MyEnum::read_fixed(&mut buf);

        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    #[cfg(feature = "experimental-write")]
    fn write_unit_enum_derived() {
        let expected = "un";

        let my_struct = MyEnum::Unit;
        let mut buf: Vec<u8> = Vec::new();
        let res = my_struct.write_fixed(&mut buf);

        assert!(res.is_ok());
        assert_eq!(to_str(buf), expected);
    }
}
