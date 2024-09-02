# Fixcol

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

let mut file = File::open("cities.txt")?;
let cities: Vec<City> = City::read_fixed_all(file).map(|res| match res {
    Ok(city) => city,
    Err(err) => {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}).collect();
```

### Multiple Record Types

Many data files contain lines corresponding to multiple types of records.
Typically the record type is indicated by the first few columns of the line.
In Fixcol we call this the *key* of the record. Multiple record types can be
decoded using an `enum` with a key annotation.

Consider a directed graph with named nodes defined in a data file like the
following.

 ```text
 NODE 001 Item A
 NODE 002 Item B
 EDGE 001 002
 ```

 This file can be parsed with an enum like the following.

 ```rust
 use fixcol::ReadFixed;

#[derive(ReadFixed)]
#[fixcol(key_width = 4)]
enum GraphItem {
    #[fixcol(key = "NODE")]
    Node {
        #[fixcol(skip = 1, width = 3)]
        id: u8,
        #[fixcol(skip = 1, width = 6)]
        name: String,
    },
    #[fixcol(key = "EDGE")]
    Edge {
        #[fixcol(skip = 1, width = 3)]
        from_id: u8,
        #[fixcol(skip = 1, width = 3)]
        to_id: u8,
    },
}
```

Please see the official documentation for complete usage guidance.

## License

Licensed under the MIT license. See: [LICENSE](LICENSE.txt).
