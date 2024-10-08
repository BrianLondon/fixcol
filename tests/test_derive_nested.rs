extern crate fixcol;

use fixcol::ReadFixed;
#[cfg(feature = "experimental-write")]
use fixcol::{WriteFixed, WriteFixedAll};

// Converted the struct Atom to AtomS and the MoleculeRow
// variant Atom to AtomV to act as a regression test where
// we did not correctly handle when those two had different
// names. i.e., Atom(Atom) worked but AtomV(AtomS) did not.

#[cfg_attr(feature = "experimental-write", derive(WriteFixed))]
#[derive(Debug, PartialEq, Eq, ReadFixed)]
struct AtomS {
    #[fixcol(width = 5, align = "right")]
    id: u16,
    #[fixcol(width = 5, align = "right")]
    molecule: u16,
    #[fixcol(skip = 1, width = 8)]
    name: String,
}

#[cfg_attr(feature = "experimental-write", derive(WriteFixed))]
#[derive(Debug, PartialEq, Eq, ReadFixed)]
#[fixcol(key_width = 3)]
enum MoleculeRow {
    #[fixcol(key = "Mol")]
    Molecule {
        #[fixcol(skip = 1, width = 5)]
        id: u16,
        #[fixcol(width = 8)]
        name: String,
    },
    #[fixcol(key = "Atm", embed = true)]
    AtomV(AtomS),
    #[fixcol(key = "Bnd")]
    Bond(#[fixcol(width = 5)] u16, #[fixcol(width = 5)] u16),
}

fn molecule_data() -> Vec<MoleculeRow> {
    fn molecule(id: u16, name: &str) -> MoleculeRow {
        MoleculeRow::Molecule { id, name: name.to_owned() }
    }

    fn atom(id: u16, molecule: u16, name: &str) -> MoleculeRow {
        let atom = AtomS { id, molecule, name: name.to_owned() };
        MoleculeRow::AtomV(atom)
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
Atm    0    0 Hydrogen
Atm    1    0 Hydrogen
Atm    2    0 Oxygen  
Bnd0    1    
Bnd1    2    
"#;

#[test]
fn read_inner() {
    // This is a regression test on a test setup error when read_nested was
    // failing for the wrong reason
    let data: AtomS = AtomS::read_fixed_str("    0    0 Hydrogen").unwrap();

    assert_eq!(
        data,
        AtomS {
            id: 0,
            molecule: 0,
            name: "Hydrogen".to_owned()
        }
    );
}

#[test]
fn read_nested() {
    let mut buf = SAMPLE_TEXT.as_bytes();
    let data: Vec<_> = MoleculeRow::read_fixed_all(&mut buf).collect();

    let actual: Vec<MoleculeRow> = data.into_iter().map(|o| o.unwrap()).collect();
    let expected = molecule_data();

    assert_eq!(actual, expected);
}

#[test]
#[cfg(feature = "experimental-write")]
fn write_nested() {
    let mut v = Vec::new();
    let res = molecule_data().write_fixed_all(&mut v);

    assert!(res.is_ok());

    let text = std::str::from_utf8(v.as_slice()).unwrap();
    assert_eq!(text, SAMPLE_TEXT);
}
