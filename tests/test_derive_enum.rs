use fixed_derive::{ReadFixed, WriteFixed};

extern crate fixed;
extern crate fixed_derive;

const SAMPLE_DATA: &'static str = r#"NODE ME
NODE NH
EDGE ME NH  327819
NODE VT
EDGE VT NH    1283
NODE MA
EDGE MA VT   83981
EDGE MA NH  904921
NODE CT
EDGE CT MA    9389
NODE RI
EDGE CT RI     412
EDGE RI MA 2948120
"#;

// TODO: Need a test case for unexpected EoF since that's usually a config error
// not actually an IO error despite being reported as such.

// TODO: "Width must be specified for all fields" should we provid an "until end of line option"?

#[derive(Debug, ReadFixed, WriteFixed, Eq, PartialEq)]
#[fixed(key_width = 4, ignore_others = true)]
enum GraphObject {
    #[fixed(key = "NODE")]
    Node(#[fixed(skip = 1, width = 2)] String),
    #[fixed(key = "EDGE")]
    Edge {
        #[fixed(skip = 1, width = 2)]
        from: String,
        #[fixed(skip = 1, width = 2)]
        to: String,
        #[fixed(skip = 1, width = 7, align = "right")]
        weight: u64,
    }, // #[fixed(key="XXXX")]
       // Blank,
}

fn node(s: &str) -> GraphObject {
    GraphObject::Node(s.to_owned())
}

fn edge(from: &str, to: &str, weight: u64) -> GraphObject {
    GraphObject::Edge {
        from: from.to_owned(),
        to: to.to_owned(),
        weight: weight,
    }
}

#[test]
fn read_enums() {
    use fixed::ReadFixed;

    let mut buf = SAMPLE_DATA.as_bytes();
    let data: Vec<_> = GraphObject::read_fixed_all(&mut buf).collect();
    println!("{:?}", data);

    let graph: Vec<GraphObject> = data.into_iter().map(|o| o.unwrap()).collect();

    let expected = vec![
        node("ME"),
        node("NH"),
        edge("ME", "NH", 327819),
        node("VT"),
        edge("VT", "NH", 1283),
        node("MA"),
        edge("MA", "VT", 83981),
        edge("MA", "NH", 904921),
        node("CT"),
        edge("CT", "MA", 9389),
        node("RI"),
        edge("CT", "RI", 412),
        edge("RI", "MA", 2948120),
    ];

    assert_eq!(graph, expected);
}

#[test]
fn write_enum() {
    use fixed::WriteFixedAll;

    use std::str::from_utf8;

    let inp = vec![
        node("ME"),
        node("NH"),
        edge("ME", "NH", 327819),
        node("VT"),
        edge("VT", "NH", 1283),
        node("MA"),
        edge("MA", "VT", 83981),
        edge("MA", "NH", 904921),
        node("CT"),
        edge("CT", "MA", 9389),
        node("RI"),
        edge("CT", "RI", 412),
        edge("RI", "MA", 2948120),
    ];

    let mut outb: Vec<u8> = Vec::new();
    let res = inp.write_fixed_all(&mut outb);

    let outs = from_utf8(outb.as_slice()).unwrap().to_string();

    assert!(res.is_ok());
    assert_eq!(outs, SAMPLE_DATA);
}
