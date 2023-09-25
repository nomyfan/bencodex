use bencodex::{BDict, BNode};
use std::{env, fs::File};

fn main() {
    let mut dict = BDict::new();
    dict.insert("bar".to_string(), "spam".into());
    dict.insert("foo".to_string(), 42.into());

    let bnode: BNode = dict.into();

    let mut file = File::create(
        env::current_dir()
            .unwrap()
            .join("examples")
            .join("bufwriter")
            .join("bufwriter-test.torrent"),
    )
    .unwrap();
    bnode.serialize(&mut file).unwrap();
}
