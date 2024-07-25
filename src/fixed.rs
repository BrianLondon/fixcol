use std::{io::{BufRead, BufReader, Lines, Read, Write}, marker::PhantomData};
use crate::error::Error;

/// Trait for writing to fixed width (column based) serialization
pub trait WriteFixed {
    /// Writes the object into the supplied buffer
    fn write_fixed<W: Write>(&self, buf: &mut W) -> Result<(), ()>;
}

/// Iterator over the deserialized lines of a fixed column file
/// 
/// Implements [`Iterator`] for `T`.
pub struct Iter<T, R>
    where T: Sized + ReadFixed, R: Read
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

impl<T: Sized + ReadFixed, R: Read>  Iter<T, R> {
    fn new(read: R) -> Self {
        Self {
            lines: BufReader::new(read).lines(),
            line: 0,
            failed: false,
            t: PhantomData,
        }
    }
}

impl<T: Sized + ReadFixed, R: Read> Iterator for Iter<T, R> {
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
                },
                Some(Ok(s)) => {
                    // TODO: think about whether we want to allow it to return the 
                    // errored line and keep going
                    match T::read_fixed_string(s) {
                        Err(Error::DataError(err)) => {
                            let err_with_line = err.with_line(self.line);
                            Some(Err(Error::DataError(err_with_line)))
                        },
                        other => Some(other)
                    }
                },
            }    
        }
    }
}

/// Trait for reading from fixed width (column based) serializaiton
/// 
/// This trait is the main entry point to using `fixed` for deserializing
/// column delimited data files. This trait is not normally implemented manually
/// but derived using the `fixed_derive` crate. The deserialization behavior
/// of individual columns is defined using the `#[fixed(...)]` annotation.
pub trait ReadFixed {
    /// Reads an instance of the object from the supplied buffer
    /// 
    /// Provides logic for deserializing an instance of the type read from a 
    /// supplied buffer. 
    /// 
    /// # Example
    /// ```
    /// # use fixed_derive::ReadFixed;
    /// # use std::fs::File;
    /// # use std::io;
    /// #[derive(ReadFixed)]
    /// struct Foo {
    ///     #[fixed(width=3)]
    ///     foo: String,
    ///     #[fixed(width=3)]
    ///     bar: String,
    /// }
    /// 
    /// # use fixed::ReadFixed;
    /// let mut buffer: &[u8] = "foobar".as_bytes();
    /// let my_foo: Foo = Foo::read_fixed(&mut buffer).unwrap();
    /// # assert_eq!(my_foo.foo, "foo".to_string());
    /// # assert_eq!(my_foo.bar, "bar".to_string());
    /// ```
    fn read_fixed<R>(buf: &mut R) -> Result<Self, Error>
        where Self: Sized, R: Read;

    /// Consumes a buffer returning objects of type `Self`
    /// 
    /// Lazily reads the entier content of `buf` returning an [`Iterator`]
    /// over deserialized objects.
    /// 
    /// # Example
    /// ```
    /// # use fixed_derive::ReadFixed;
    /// # use std::fs::File;
    /// # use std::io;
    /// #[derive(ReadFixed)]
    /// struct MyType {
    ///     // ...
    /// }
    /// 
    /// # use fixed::ReadFixed;
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
        where Self: Sized, R: Read
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
    /// # use fixed_derive::ReadFixed;
    /// # use fixed::FixedDeserializer;
    /// # use fixed::FieldDescription;
    /// #[derive(ReadFixed)]
    /// struct Point {
    ///     #[fixed(width=3, align="right")]
    ///     x: u8,
    ///     #[fixed(width=3, align="right")]
    ///     y: u8,
    /// }
    /// 
    /// # use fixed::ReadFixed;
    /// let point = Point::read_fixed_str(" 42  7").unwrap();
    /// assert_eq!(point.x, 42);
    /// assert_eq!(point.y, 7)
    /// ```
    /// 
    /// It can also be useful to pull directly from slices.
    /// ```
    /// # use fixed_derive::ReadFixed;
    /// # use fixed::FixedDeserializer;
    /// # use fixed::FieldDescription;
    /// # #[derive(ReadFixed)]
    /// # struct Point {
    /// #     #[fixed(width=3)]
    /// #     x: u8,
    /// #     #[fixed(width=3)]
    /// #     y: u8,
    /// # }
    /// # use fixed::ReadFixed;
    /// let s = ">>12361 <<";
    /// let point = Point::read_fixed_str(&s[2..8]).unwrap();
    /// 
    /// assert_eq!(point.x, 123);
    /// assert_eq!(point.y, 61);
    /// ```
    fn read_fixed_str(s: &str) -> Result<Self, Error> 
        where Self: Sized
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
    /// # use fixed_derive::ReadFixed;
    /// # use fixed::FixedDeserializer;
    /// # use fixed::FieldDescription;
    /// #[derive(ReadFixed)]
    /// struct Point {
    ///     #[fixed(width=3, align="right")]
    ///     x: u8,
    ///     #[fixed(width=3, align="right")]
    ///     y: u8,
    /// }
    /// 
    /// # use fixed::ReadFixed;
    /// let s = String::from(" 42  7");
    /// let point = Point::read_fixed_string(s).unwrap();
    /// assert_eq!(point.x, 42);
    /// assert_eq!(point.y, 7)
    /// ```
    fn read_fixed_string(s: String) -> Result<Self, Error> 
        where Self: Sized
    {
        let mut bytes = s.as_bytes();
        Self::read_fixed(&mut bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Foo {
        foo: String,
    }

    impl ReadFixed for Foo {
        fn read_fixed<R>(buf: &mut R) -> Result<Self, Error>
            where Self: Sized, R: Read {
            
            let mut s: String = String::new();
            buf.read_to_string(&mut s)?;

            Ok(Self { foo: s })
        }
    }

    #[test]
    fn read_fixed_str() {
        let foo = Foo::read_fixed_str("bar");
        assert_eq!(foo.unwrap(), Foo{ foo: "bar".to_string()});
    }

    #[test]
    fn read_fixed_string() {
        let s: String = "bar".to_string();
        let foo = Foo::read_fixed_string(s);
        assert_eq!(foo.unwrap(), Foo{ foo: "bar".to_string()});
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

}
