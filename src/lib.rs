use std::fmt::Display;

type BList = Vec<BNode>;
type BMap = std::collections::BTreeMap<String, BNode>;

pub enum BInt {
    Int32(i32),
    Int64(i64),
}

pub enum BNode {
    Int(BInt),
    Str(String),
    List(BList),
    Map(BMap),
}

// Impl Display trait
impl Display for BInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            BInt::Int32(i) => write!(f, "i{}e", i),
            BInt::Int64(i) => write!(f, "i{}e", i),
        }
    }
}

impl Display for BNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            BNode::Int(i) => Display::fmt(&i, f),
            BNode::Str(s) => write!(f, "{}:{}", &s.len(), &s),
            BNode::List(l) => {
                write!(f, "l")?;
                for n in l {
                    Display::fmt(&n, f)?;
                }
                write!(f, "e")
            }
            BNode::Map(m) => {
                write!(f, "d")?;
                for n in m {
                    let key = &n.0;
                    let val = &n.1;
                    write!(f, "{}:{}", key.len(), key)?;
                    Display::fmt(val, f)?;
                }

                write!(f, "e")
            }
        }
    }
}
// End impl Display trait

pub fn parse(input: &str) -> BNode {
    let (node, len) = internal_parse(input);
    if len != input.len() {
        panic!("Invalid input");
    }
    node
}

fn internal_parse(input: &str) -> (BNode, usize) {
    let desc = input.chars().next().unwrap();
    match desc {
        'l' => parse_list(input),
        'd' => parse_map(input),
        '0'..='9' => parse_string(input),
        'i' => parse_int(input),
        _ => panic!("invalid input"),
    }
}

fn parse_int(input: &str) -> (BNode, usize) {
    let mut val: i64 = 0;
    let mut positive = true;
    let mut next: usize = 1;
    for c in input.chars().skip(1) {
        next = next + 1;
        if c == 'e' {
            break;
        }
        if c == '-' {
            positive = false;
        }
        val = val * 10 + (c.to_digit(10).expect("Invalid integer") as i64);
    }

    if !positive {
        val = -val;
    }

    if val > std::i32::MAX.into() || val < std::i32::MIN.into() {
        (BNode::Int(BInt::Int64(val)), next)
    } else {
        (BNode::Int(BInt::Int32(val as i32)), next)
    }
}

fn parse_string(input: &str) -> (BNode, usize) {
    let mut len = 0;
    let mut next = 1;
    for c in input.chars() {
        if c == ':' {
            break;
        }
        next = next + 1;
        len = len * 10 + (c.to_digit(10).expect("Bad string length is given"));
    }

    let mut ret = String::new();
    ret.push_str(&input[next..(next + len as usize)]);
    next = next + len as usize;

    (BNode::Str(ret), next)
}

fn parse_list(input: &str) -> (BNode, usize) {
    let mut nodes = vec![];
    let mut next = 1;
    loop {
        let (node, n) = internal_parse(&input[next..]);
        next = next + n;
        nodes.push(node);
        if "e" == &input[next..next + 1] {
            next = next + 1;
            break;
        }
    }

    (BNode::List(nodes), next)
}

fn parse_map(input: &str) -> (BNode, usize) {
    let mut map = BMap::new();
    let mut next = 1;
    loop {
        let (key_node, n) = internal_parse(&input[next..]);
        next = next + n;
        let key = match key_node {
            BNode::Str(s) => s,
            _ => panic!("Dictionary key's type should be String"),
        };

        let (val_node, n) = internal_parse(&input[next..]);
        next = next + n;
        map.insert(key, val_node);

        if "e" == &input[next..next + 1] {
            next = next + 1;
            break;
        }
    }

    (BNode::Map(map), next)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_bint_display() {
        let bi32 = crate::BNode::Int(crate::BInt::Int32(32));
        println!("{}", bi32);
    }

    #[test]
    fn test_string_display() {
        let bstr = crate::BNode::Str("BNode::Str Display trait".to_string());
        println!("{}", bstr);
    }

    #[test]
    fn test_type() {
        // primitive type
        use crate::BInt;
        use crate::BNode;
        let _node = BNode::Int(BInt::Int32(32));
        let _str_node = BNode::Str("str".to_string());

        // list
        use crate::BList;
        let node_vec: BList = vec![BNode::Int(BInt::Int32(32)), BNode::Int(BInt::Int64(64))];

        // dictionary
        let mut map = std::collections::HashMap::new();
        map.insert("key", BNode::List(node_vec));
    }

    #[test]
    fn test_primitive_int() {
        use crate::{BInt, BNode};
        let (bi32, n1) = crate::parse_int("i123e");
        match bi32 {
            BNode::Int(v) => match v {
                BInt::Int32(i) => {
                    assert_eq!(i, 123);
                    assert_eq!("i123e".len(), n1);
                }
                _ => panic!("wrong type"),
            },
            _ => panic!("wrong type"),
        }
        let (bi64, n2) = crate::parse_int("i2147483648e");
        if let BNode::Int(v) = bi64 {
            if let BInt::Int64(i) = v {
                assert_eq!(i, 2147483648);
                assert_eq!("i2147483648e".len(), n2);
            }
        }
    }

    #[test]
    fn test_primitive_string() {
        use crate::BNode;
        let (node, next) = crate::parse_string("5:hello");
        if let BNode::Str(v) = &node {
            assert_eq!("hello", &v[..]);
            assert_eq!(7, next);
        }
    }

    #[test]
    fn test_list() {
        use crate::BInt;
        use crate::BNode;

        let (node, next) = crate::parse_list("l4:spami42ee");
        assert_eq!("l4:spami42ee".len(), next);

        if let BNode::List(list) = node {
            assert_eq!(2, list.len());

            if let BNode::Str(s) = &list[0] {
                assert_eq!("spam", s);
            }

            if let BNode::Int(i) = &list[1] {
                if let BInt::Int32(int32) = i {
                    assert_eq!(&42, int32);
                }
            }
        }
    }

    #[test]
    fn test_nested_list() {
        let input = "ll5:helloe4:spami42ee";
        let (node, next) = crate::parse_list(&input);
        assert_eq!("ll5:helloe4:spami42ee".len(), next);
        assert_eq!(&format!("{}", node), &input);
    }

    #[test]
    fn test_map() {
        let input =
            "d8:announce41:http://bttracker.debian.org:6969/announce13:creation datei15739038104ee";
        let len = input.len();
        let (node, next) = crate::internal_parse(&input);
        assert_eq!(len, next);
        assert_eq!(&format!("{}", node), &input);
    }

    #[test]
    fn test_nested_map() {
        let input = r#"d8:announce41:http://bttracker.debian.org:6969/announce7:comment35:"Debian CD from cdimage.debian.org"13:creation datei1573903810e9:httpseedsl145:https://cdimage.debian.org/cdimage/release/10.2.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-10.2.0-amd64-netinst.iso145:https://cdimage.debian.org/cdimage/archive/10.2.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-10.2.0-amd64-netinst.isoe4:infod6:lengthi351272960e4:name31:debian-10.2.0-amd64-netinst.iso12:piece lengthi262144eee"#;
        let len = input.len();
        let (node, next) = crate::internal_parse(&input);

        assert_eq!(len, next);
        assert_eq!(&format!("{}", node), &input);
    }
}
