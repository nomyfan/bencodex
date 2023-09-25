use std::{
    env,
    fs::File,
    io::{BufReader, Bytes, Read},
};

struct Adapter {
    bytes: Bytes<BufReader<File>>,
}

impl Iterator for Adapter {
    type Item = u8;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.bytes.next() {
            Some(Ok(v)) => Some(v),
            _ => None,
        }
    }
}

fn main() {
    let f = File::open(
        env::current_dir()
            .unwrap()
            .join("examples")
            .join("bufreader")
            .join("bufreader-test.torrent"),
    )
    .unwrap();
    let reader = BufReader::new(f);

    let mut adapter = Adapter {
        bytes: reader.bytes(),
    };
    let bnode = bencodex::parse(&mut adapter).unwrap();
    let dict = bnode.as_dict().unwrap();
    assert_eq!(
        dict.get("bar").unwrap().as_bytes().unwrap(),
        "spam".as_bytes()
    );
    assert_eq!(dict.get("foo").unwrap().as_integer().unwrap(), &42);
}
