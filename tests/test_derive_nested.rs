extern crate fixed;
extern crate fixed_derive;

use fixed_derive::{ReadFixed, WriteFixed};

#[derive(Debug, PartialEq, Eq, ReadFixed, WriteFixed)]
struct Atom {
    #[fixed(width = 5, align = "right")]
    id: u16,
    #[fixed(width = 5, align = "right")]
    molecule: u16,
    #[fixed(skip = 1, width = 8)]
    name: String,
}

// #[derive(Debug, PartialEq, Eq, ReadFixed, WriteFixed)]
#[derive(Debug, PartialEq, Eq, ReadFixed)]
#[fixed(key_width = 3)]
enum MoleculeRow {
    #[fixed(key = "Mol")]
    Molecule { 
        #[fixed(skip = 1, width = 5)]
        id: u16,
        #[fixed(width = 8)]
        name: String 
    },
    #[fixed(key = "Atm", embed = true)]
    Atom(Atom),
    #[fixed(key = "Bnd")]
    Bond(#[fixed(width = 5)] u16, #[fixed(width = 5)] u16),
}


fn molecule_data() -> Vec<MoleculeRow> {
    fn molecule(id: u16, name: &str) -> MoleculeRow {
        MoleculeRow::Molecule { id, name: name.to_owned() }
    }

    fn atom(id: u16, molecule: u16, name: &str) -> MoleculeRow {
        let atom = Atom { id, molecule, name: name.to_owned() };
        MoleculeRow::Atom(atom)
    }

    fn bond(a: u16, b: u16) -> MoleculeRow {
        MoleculeRow::Bond(a, b)
    }

    vec![
        molecule(0, "Water"),
        atom(0, 0, "Hydrogen"),
        atom(1, 0, "Hydrogen"),
        atom(2, 0, "Oxygen"),
        bond(0, 1),
        bond(1, 2),
    ] 
}

const SAMPLE_TEXT: &'static str = r#"Mol 0    Water   
Atm    0    0Hydrogen
Atm    1    0Hydrogen
Atm    1    0Oxygen  
Bnd0    1            
Bnd1    2            
"#;

#[test]
fn read_inner() {
    use fixed::ReadFixed;

    let mut buf = "    0    0Hydrogen".as_bytes();
    let data: Atom = Atom::read_fixed(&mut buf).unwrap();

    assert_eq!(data, Atom{ id: 0, molecule: 0, name: "Hydrogen".to_owned() });
}

#[test]
fn read_nested() {
    use fixed::ReadFixed;

    let mut buf = SAMPLE_TEXT.as_bytes();
    let data: Vec<_> = MoleculeRow::read_fixed_all(&mut buf).collect();

    let actual: Vec<MoleculeRow> = data.into_iter().map(|o| {println!("{:?}", o); o.unwrap()}).collect();
    let expected = molecule_data();

    assert_eq!(actual, expected);
}
