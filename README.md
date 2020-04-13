# bencode

A bencode parser.

## Example
```rust
use bencodex::BNode;

fn main() {
    unmarshal();
    println!("-----------------");
    marshal();
}

fn unmarshal() {
    let b = "d3:inti233e3:lstl7:bencodeee";
    let result = bencodex::parse(&b);

    match result {
        Ok(node) => {
            let map = node.as_map().unwrap();
            let int = map.get("int").unwrap().as_int().unwrap();
            println!("Int = {}", int);
            let lst = map.get("lst").unwrap().as_list().unwrap();
            println!("There're {} values in the list", lst.len());
            println!(
                "The first value in the list is `{}`",
                &lst[0].as_str().unwrap()
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
        BNode::List(vec![BNode::Str("bencode".to_string())]),
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