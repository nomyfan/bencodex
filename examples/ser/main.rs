use bencodex::{BDict, BNode};

fn main() {
    let mut dict = BDict::new();
    dict.insert("bar".to_string(), BNode::Bytes("spam".bytes().collect()));
    dict.insert("foo".to_string(), BNode::Integer(42));

    let bnode = BNode::Dict(dict);

    assert_eq!("d3:bar4:spam3:fooi42ee", bnode.to_string())
}
