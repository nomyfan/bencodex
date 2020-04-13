use std::fmt::Display;

pub type BList = Vec<BNode>;
pub type BMap = std::collections::BTreeMap<String, BNode>;

pub enum BNode {
    Int(i64),
    Str(String),
    List(BList),
    Map(BMap),
}

impl BNode {
    pub fn as_int(&self) -> Option<i64> {
        if let BNode::Int(int) = self {
            return Some(*int);
        }
        None
    }

    pub fn as_str(&self) -> Option<&str> {
        if let BNode::Str(s) = self {
            return Some(s);
        }
        None
    }

    pub fn as_list(&self) -> Option<&BList> {
        if let BNode::List(lst) = self {
            return Some(lst);
        }
        None
    }

    pub fn as_map(&self) -> Option<&BMap> {
        if let BNode::Map(m) = self {
            return Some(m);
        }
        None
    }
}

impl Display for BNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            BNode::Int(i) => write!(f, "i{}e", i),
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

/// Parse bencoded string
///
/// # Examples
/// ```
/// let b = "d3:inti233e3:lstl7:bencodeee";
/// let result = bencodex::parse(&b);
/// match result {
///     Ok(node) => {
///         let map = node.as_map().unwrap();
///         let int = map.get("int").unwrap().as_int().unwrap();
///         println!("Int = {}", int);
///         let lst = map.get("lst").unwrap().as_list().unwrap();
///         println!("There're {} values in the list", lst.len());
///         println!(
///             "The first value in the list is `{}`",
///             &lst[0].as_str().unwrap()
///         );
///     }
///     Err(e) => panic!(e),
/// }
/// ```
pub fn parse(input: &str) -> Result<BNode, String> {
    let (node, len) = internal_parse(input)?;
    if len != input.len() {
        return Err("Invalid input".to_string());
    }
    Ok(node)
}

fn internal_parse(input: &str) -> Result<(BNode, usize), String> {
    let desc = input.chars().next().unwrap();
    match desc {
        'l' => parse_list(input),
        'd' => parse_map(input),
        '0'..='9' => parse_string(input),
        'i' => parse_int(input),
        _ => Err("Undefined stopword".to_string()),
    }
}

fn parse_int(input: &str) -> Result<(BNode, usize), String> {
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
        match c.to_digit(10) {
            Some(d) => val = val * 10 + d as i64,
            None => return Err("Invalid integer".to_string()),
        }
    }

    if !positive {
        val = -val;
    }

    Ok((BNode::Int(val), next))
}

fn parse_string(input: &str) -> Result<(BNode, usize), String> {
    let mut len = 0;
    let mut next = 1;
    for c in input.chars() {
        if c == ':' {
            break;
        }
        next = next + 1;
        match c.to_digit(10) {
            Some(d) => len = len * 10 + d,
            None => return Err("Bad string length is given".to_string()),
        }
    }

    let mut ret = String::new();
    ret.push_str(&input[next..(next + len as usize)]);
    next = next + len as usize;

    Ok((BNode::Str(ret), next))
}

fn parse_list(input: &str) -> Result<(BNode, usize), String> {
    let mut nodes = vec![];
    let mut next = 1;
    loop {
        let (node, n) = internal_parse(&input[next..])?;
        next = next + n;
        nodes.push(node);
        if "e" == &input[next..next + 1] {
            next = next + 1;
            break;
        }
    }

    Ok((BNode::List(nodes), next))
}

fn parse_map(input: &str) -> Result<(BNode, usize), String> {
    let mut map = BMap::new();
    let mut next = 1;
    loop {
        let (key_node, n) = internal_parse(&input[next..])?;
        next = next + n;
        let key = match key_node {
            BNode::Str(s) => s,
            _ => return Err("Dictionary key's type should be String".to_string()),
        };

        let (val_node, n) = internal_parse(&input[next..])?;
        next = next + n;
        map.insert(key, val_node);

        if "e" == &input[next..next + 1] {
            next = next + 1;
            break;
        }
    }

    Ok((BNode::Map(map), next))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_bint_display() {
        let bi32 = crate::BNode::Int(32);
        assert_eq!(&format!("{}", bi32), "i32e");

        let neg_bi32 = crate::BNode::Int(-32);
        assert_eq!(&format!("{}", neg_bi32), "i-32e");
    }

    #[test]
    fn test_string_display() {
        let bstr = crate::BNode::Str("str".to_string());
        assert_eq!(&format!("{}", bstr), "3:str");
    }

    #[test]
    fn test_primitive_int() {
        use crate::BNode;
        let result = crate::parse_int("i2147483648e");
        match result {
            Ok((bint, n1)) => match bint {
                BNode::Int(i) => {
                    assert_eq!(i, 2147483648);
                    assert_eq!("i2147483648e".len(), n1);
                }
                _ => panic!("Wrong type"),
            },
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_primitive_string() {
        use crate::BNode;
        let result = crate::parse_string("5:hello");
        match result {
            Ok((node, next)) => match node {
                BNode::Str(v) => {
                    assert_eq!("hello", &v[..]);
                    assert_eq!(7, next);
                }
                _ => panic!("Wrong type"),
            },
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_list() {
        let input = "l4:spami42ee";
        let len = input.len();
        let result = crate::parse_list(&input);
        match result {
            Ok((node, next)) => {
                assert_eq!(len, next);
                assert_eq!(&format!("{}", node), &input);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_nested_list() {
        let input = "ll5:helloe4:spami42ee";
        let len = input.len();
        let result = crate::parse_list(&input);

        match result {
            Ok((node, next)) => {
                assert_eq!(len, next);
                assert_eq!(&format!("{}", node), &input);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_map() {
        let input =
            "d8:announce41:http://bttracker.debian.org:6969/announce13:creation datei15739038104ee";
        let len = input.len();
        let result = crate::internal_parse(&input);

        match result {
            Ok((node, next)) => {
                assert_eq!(len, next);
                assert_eq!(&format!("{}", node), &input);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_nested_map() {
        let input = r#"d8:announce41:http://bttracker.debian.org:6969/announce7:comment35:"Debian CD from cdimage.debian.org"13:creation datei1573903810e9:httpseedsl145:https://cdimage.debian.org/cdimage/release/10.2.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-10.2.0-amd64-netinst.iso145:https://cdimage.debian.org/cdimage/archive/10.2.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-10.2.0-amd64-netinst.isoe4:infod6:lengthi351272960e4:name31:debian-10.2.0-amd64-netinst.iso12:piece lengthi262144eee"#;
        let len = input.len();
        let result = crate::internal_parse(&input);

        match result {
            Ok((node, next)) => {
                assert_eq!(len, next);
                assert_eq!(&format!("{}", node), &input);
            }
            Err(e) => panic!(e),
        }
    }
}
