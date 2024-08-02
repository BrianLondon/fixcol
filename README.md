# Fixed

A library for reading fixed with / column delimited data files.

## Wishlist of new features

 - Enum variants wrapping a single struct
 - Fixed column offsets
 - Error messages for writing operations
 - Strict mode
 

Also... do this: https://stackoverflow.com/questions/54392702/how-to-report-errors-in-a-procedural-macro-using-the-quote-macro
    It seems like we just make all of the inner methods return Results with an error type that has a message and a span
    If we see that when we get to rendering then the error takes precedence.
    This might not be a release 1 thing.
