# Fixed

A library for reading fixed width / column delimited data files.

## Wishlist of new features

 - Fixed column offsets
 - Error messages for writing operations
 - Strict mode
 - Add an option for padding enum variants to all be the same length
    - Also support shorter than expected lines gracefully
 - Make param list data rather than code to support dynamic lists of
   valid parameters.
 - Allow a function based custom deserialization on individual columns

Also... do this: https://stackoverflow.com/questions/54392702/how-to-report-errors-in-a-procedural-macro-using-the-quote-macro
    It seems like we just make all of the inner methods return Results with an error type that has a message and a span
    If we see that when we get to rendering then the error takes precedence.
    This might not be a release 1 thing.
