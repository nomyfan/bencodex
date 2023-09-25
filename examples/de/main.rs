fn main() {
    let b = "d3:inti233e3:lstl7:bencodeee";
    let node = bencodex::parse(&mut b.bytes()).unwrap();

    let dict = node.as_dict().unwrap();
    let int = dict.get("int").unwrap().as_integer().unwrap();

    assert_eq!(int, &233);
    let list = dict.get("lst").unwrap().as_list().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(
        list.get(0).unwrap().as_bytes().unwrap(),
        "bencode".as_bytes()
    );
}
