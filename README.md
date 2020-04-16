# bencodex

A bencode parser.

## Example

## Plain text
```rust
use bencodex::BNode;

fn main() {
    unmarshal();
    println!("-----------------");
    marshal();
}

fn unmarshal() {
    let b = "d3:inti233e3:lstl7:bencodeee";
    let result = bencodex::parse(&mut b.bytes());

    match result {
        Ok(node) => {
            let map = node.as_map().unwrap();
            let int = map.get("int").unwrap().as_int().unwrap();
            println!("Int = {}", int);
            let lst = map.get("lst").unwrap().as_list().unwrap();
            println!("There're {} values in the list", lst.len());
            println!(
                "The first value in the list is `{}`",
                &lst[0].as_string().unwrap()
            );
        }
        Err(e) => panic!(e),
    }
}

fn marshal() {
    let mut map = std::collections::BTreeMap::new();
    map.insert("int".to_string(), BNode::Int(2333));
    map.insert(
        "lst".to_string(),
        BNode::List(vec![BNode::Str("bencode".bytes().collect::<Vec<u8>>())]),
    );

    println!("{}", &BNode::Map(map));
}
```

Output
```
Int = 233
There're 1 values in the list
The first value in the list is `bencode`
-----------------
d3:inti2333e3:lstl7:bencodeee
```

### BitTorrent file
Iterator adapter
```rust
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
```
Unmarshal
```rust
fn unmarshal_from_bitorrent_file() -> io::Result<()> {
    let f = File::open("Ps.torrent")?;
    let reader = BufReader::new(f);

    let mut adapter = Adapter {
        bytes: reader.bytes(),
    };
    if let Ok(BNode::Map(map)) = bencodex::parse(&mut adapter) {
        for k in map.keys() {
            println!("{}", k);
        }
    }

    Ok(())
}
```
Output
```
announce
announce-list
created by
creation date
encoding
info
```