# bencodex

A bencode parser.

## Get Started

```rust
use bencodex::{BDict, BNode};

let mut dict = BDict::new();
dict.insert("bar".to_string(), "spam".into());
dict.insert("foo".to_string(), 42.into());

let bnode = BNode::Dict(dict);

let mut file = File::create(
  env::current_dir().unwrap().join("name.torrent")
).unwrap();
bnode.serialize(&mut file).unwrap();
```
