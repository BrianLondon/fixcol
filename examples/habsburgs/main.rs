use std::{fs::File, io};
use std::path::Path;

use alg::coi_for_data_set;
use fixcol::{ReadFixed, WriteFixed, WriteFixedAll};

mod alg;

#[derive(Debug, Eq, PartialEq, ReadFixed)]
#[fixcol(key_width = 2, strict = false)]
enum RelationType {
    #[fixcol(key = "PC")]
    ParentChild,

    #[fixcol(key = "SP")]
    Spouse,
}

#[derive(Debug, Eq, PartialEq, ReadFixed)]
#[fixcol(key_width = 1)]
enum Record {
    #[fixcol(key = "P")]
    Person{
        #[fixcol(width = 3)]
        id: u8,
        #[fixcol(width = 11, align = "right")]
        name: String,
        #[fixcol(skip = 1, width = 4)]
        regnal_number: String,
        #[fixcol(width = 4)]
        birth: u16,
        #[fixcol(skip = 1, width = 4, strict = false)]
        death: u16,
    },

    #[fixcol(key = "R")]
    Relation {
        #[fixcol(skip = 1, width = 2)]
        rel_type: RelationType,
        #[fixcol(skip = 1, width = 3)]
        from: u8,
        #[fixcol(width = 3, strict = false)]
        to: u8,
    },
}

#[derive(Debug, WriteFixed)]
struct OutputRecord {
    #[fixcol(width = 6)]
    coi: f32,
    #[fixcol(skip = 1, width = 30)]
    name: String,
}

pub fn main() {
    let path = Path::new(file!())
        .parent()
        .unwrap()
        .join("input.txt")
        .canonicalize()
        .unwrap();

    let file = File::open(path).unwrap();

    // Read the data file
    let records: Vec<Record> = Record::read_fixed_all(file)
        .map(|result| match result {
            Ok(record) => record,
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        })
        .collect();

    // Run the coi calculation
    let results = coi_for_data_set(records).into_iter().filter(|r| r.coi > 0.0);

    // Write the serialized output to STDOUT
    let mut stdout = io::stdout();
    let _ = results.write_fixed_all(&mut stdout);
}
