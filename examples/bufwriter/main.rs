use bencodex::{BDict, BNode};
use std::{env, fs::File};

fn main() {
    let mut dict = BDict::new();
    dict.insert("bar".to_string(), BNode::Bytes("spam".bytes().collect()));
    dict.insert("foo".to_string(), BNode::Integer(42));

    let bnode = BNode::Dict(dict);

    let mut file = File::create(
        env::current_dir()
            .unwrap()
            .join("examples")
            .join("bufwriter")
            .join("bufwriter-test.torrent"),
    )
    .unwrap();
    bnode.marshal(&mut file).unwrap();
}
