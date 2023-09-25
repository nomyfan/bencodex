fn main() {
    let b = "d3:inti233e3:lstl7:bencodeee";
    let result = bencodex::parse(&mut b.bytes());

    match result {
        Ok(node) => {
            let map = node.as_dict().unwrap();
            let int = map.get("int").unwrap().as_integer().unwrap();

            assert_eq!(int, &233);
            let list = map.get("lst").unwrap().as_list().unwrap();
            assert_eq!(list.len(), 1);
            assert_eq!(
                list.get(0).unwrap().as_bytes().unwrap(),
                "bencode".as_bytes()
            );
        }
        Err(e) => panic!("{:?}", e),
    }
}
