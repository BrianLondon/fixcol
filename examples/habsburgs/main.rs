use std::fs::File;
use std::path::Path;

use fixed::ReadFixed;

#[derive(Debug, Eq, PartialEq, ReadFixed)]
#[fixed(key_width = 2, strict = false)]
enum RelationType {
    #[fixed(key = "PC")]
    ParentChild,

    #[fixed(key = "HW")]
    HusbandWife,
}

#[derive(Debug, Eq, PartialEq, ReadFixed)]
#[fixed(key_width = 1)]
enum Record {
    #[fixed(key = "P")]
    Person{
        #[fixed(width = 3)]
        id: u8,
        #[fixed(width = 11, align = "right")]
        name: String,
        #[fixed(skip = 1, width = 4)]
        regnal_number: String,
        #[fixed(width = 4)]
        birth: u16,
        #[fixed(skip = 1, width = 4, strict = false)]
        death: u16,
    },

    #[fixed(key = "R")]
    Relation {
        #[fixed(skip = 1, width = 2)]
        rel_type: RelationType,
        #[fixed(skip = 1, width = 3)]
        from: u8,
        #[fixed(width = 3, strict = false)]
        to: u8,
    },
}

pub fn main() {
    let path = Path::new(file!())
        .parent()
        .unwrap()
        .join("input.txt")
        .canonicalize()
        .unwrap();

    let file = File::open(path).unwrap();

    let records: Vec<Record> = Record::read_fixed_all(file)
        .map(|result| match result {
            Ok(record) => {
                println!("{:?}", record);
                record
            }
            Err(err) => {
                println!("{}", err);
                std::process::exit(1);
            }
        })
        .collect();

    for record in records {
        println!("{:?}", record);
    }
}
