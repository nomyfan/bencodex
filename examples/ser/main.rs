use bencodex::{BDict, BNode};

fn main() {
    let mut dict = BDict::new();
    dict.insert("bar".to_string(), "spam".into());
    dict.insert("foo".to_string(), 42.into());

    let bnode: BNode = dict.into();

    let mut buf = vec![];
    bnode.serialize(&mut buf).unwrap();
    assert_eq!("d3:bar4:spam3:fooi42ee".as_bytes(), buf)
}
