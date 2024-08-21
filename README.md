# Fixed

A library for reading fixed width / column delimited data files.

## Strict Mode

We'll start by enabling it only on the field level and then allowing cascades
as a future enhancement. There's currently no parsing of attributes on the
`struct` level, so that also provides an impediment to the cascade behavior.

What strict should enable
 - require last field of line to be full length when reading
 - require written `Full` aligned text columns to be the correct length
 - require `Left` and `Right` aligned text columns to not overflow <!-- TODO: need test coverage for this -->
 - require unread columns to contain only whitespace
 - require no whitespace in numeric `Full` columns
 - left aligned fields cannot start with white space
 - right aligned fields cannot end with white space
 - error on integer width overflow on write <!-- TODO: need test coverage for this -->

## Wishlist of new features

 - Fixed column offsets
 - Error messages for writing operations
 - Strict mode
 - Add an option for padding enum variants to all be the same length
    - Also support shorter than expected lines gracefully
 - Make param list data rather than code to support dynamic lists of
   valid parameters.
 - Allow a function based custom deserialization on individual columns
 - Clear error messages of location of error on read errors
