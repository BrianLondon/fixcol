# Fixed

A library for reading fixed width / column delimited data files.

## Basic Usage

Consider the following data file:
```text
Tokyo       13515271   35.689  139.692
Delhi       16753235   28.610   77.230
Shanghai    24870895   31.229  121.475
São Paulo   12252023  -23.550  -46.333
Mexico City  9209944   19.433  -99.133
```

We can create a basic data structure corresponding to the records in the file
and then read the data file as shown.

```rust
use fixcol::ReadFixed;
use std::fs::File;

#[derive(ReadFixed)]
struct City {
    #[fixcol(width = 12)]
    name: String,
    #[fixcol(width = 8, align = "right")]
    population: u64,
    #[fixcol(skip = 1, width = 8, align = "right")]
    lat: f32,
    #[fixcol(skip = 1, width = 8, align = "right")]
    lon: f32,
}

let mut file = File::open("cities.txt");
let cities: Vec<City> = City::read_fixed_all(file).map(|res| match res {
    Ok(city) => city,
    Err(err) => {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}).collect();
```

Please see the official documentation for complete usage guidance.

 <!-- 
 
  TODO: need test coverage for:
  require `Left` and `Right` aligned text columns to not overflow in strict mode

  TODO: need test coverage for:
  error on overflow on write (esp. integers)
  -->


## Wishlist of new features

 - Fixed column offsets
 - Better error messages for writing operations
 - Make param list data rather than code to support dynamic lists of
   valid parameters.
 - Allow a function based custom deserialization on individual columns
 - Clear error messages of location of error on read errors
 - Enable the `ignore_others` parameter

## License

Licensed under the MIT license. See: [LICENSE.txt].
