pub type BList = Vec<BNode>;
pub type BDict = std::collections::BTreeMap<String, BNode>;

pub enum BNode {
    Int(i64),
    Stream(Vec<u8>),
    List(BList),
    Dict(BDict),
}

impl BNode {
    pub fn marshal(&self, buf: &mut Vec<u8>) {
        match self {
            BNode::Int(i) => {
                buf.push(b'i');
                push_all(i.to_string().as_bytes(), buf);
                buf.push(b'e');
            }
            BNode::Stream(s) => {
                push_all(s.len().to_string().as_bytes(), buf);
                buf.push(b':');
                push_all(s, buf);
            }
            BNode::List(l) => {
                buf.push(b'l');
                for bn in l {
                    bn.marshal(buf);
                }
                buf.push(b'e');
            }
            BNode::Dict(m) => {
                buf.push(b'd');
                for (k, v) in m {
                    push_all(k.len().to_string().as_bytes(), buf);
                    buf.push(b':');
                    push_all(k.as_bytes(), buf);
                    v.marshal(buf);
                }
                buf.push(b'e');
            }
        }
    }
}

#[inline]
fn push_all(bytes: &[u8], buf: &mut Vec<u8>) {
    for x in bytes {
        buf.push(*x);
    }
}

/**
https://en.wikipedia.org/wiki/Bencode
*/
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Token {
    IntStart,
    IntEnd,
    ListStart,
    ListEnd,
    DictStart,
    DictEnd,
    StreamLength(i64),
    StreamColon,
    EOF,
}

pub struct Lexer<'a, T>
where
    T: Iterator<Item = u8>,
{
    stream: &'a mut T,
    position: i64,
    cached_byte: Option<u8>,
    cached_token: Option<Token>,

    token_stack: Vec<Token>,
    current_token: Option<Token>,
}

impl<'a, T> Lexer<'a, T>
where
    T: Iterator<Item = u8>,
{
    fn new(stream: &'a mut T) -> Lexer<'a, T> {
        Lexer {
            stream,
            position: -1,
            cached_byte: None,
            cached_token: None,

            token_stack: vec![],
            current_token: None,
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        let next = match self.cached_byte {
            Some(_) => self.cached_byte.take(),
            None => {
                self.position += 1;
                self.stream.next()
            }
        };

        next
    }

    fn read_i64_before(&mut self, init: i64, symbol: u8) -> i64 {
        let mut num = init;
        let mut sign = 1i64;
        let mut read = 0;

        let mut meet = self.next_byte();
        while let Some(x) = meet {
            read += 1;

            match x {
                b'0'..=b'9' => num = num * 10 + (x - b'0') as i64,
                b'-' => match sign {
                    -1 if read != 1 => {
                        panic!("minus can only appear in the front of the number")
                    }
                    _ => sign = -1,
                },
                b if b == symbol => {
                    self.cached_byte = Some(symbol);
                    return sign * num;
                }
                _ => panic!("invalid number"),
            }

            meet = self.next_byte();
        }

        panic!("invalid number");
    }

    fn read_nbytes(&mut self, len: usize) -> Vec<u8> {
        let mut ret = Vec::with_capacity(len);

        for _ in 0..len {
            match self.next_byte() {
                Some(byte) => ret.push(byte),
                None => panic!("Stream is expected to be longer"),
            }
        }

        ret
    }

    fn next_token(&mut self) -> Token {
        if let Some(token) = self.cached_token.take() {
            return token;
        }

        match self.next_byte() {
            Some(unknown) => match unknown {
                b'i' => {
                    self.current_token = Some(Token::IntStart);
                    self.token_stack.push(Token::IntStart);

                    Token::IntStart
                }
                b'l' => {
                    self.current_token = Some(Token::ListStart);
                    self.token_stack.push(Token::ListStart);

                    Token::ListStart
                }
                b'd' => {
                    self.current_token = Some(Token::DictStart);
                    self.token_stack.push(Token::DictStart);

                    Token::DictStart
                }
                b'e' => match &self.token_stack.pop() {
                    Some(Token::IntStart) => {
                        self.current_token = None;

                        Token::IntEnd
                    }
                    Some(Token::ListStart) => {
                        self.current_token = None;

                        Token::ListEnd
                    }
                    Some(Token::DictStart) => {
                        self.current_token = None;

                        Token::DictEnd
                    }
                    _ => panic!("`e` should be the end of integer, list and dictionary"),
                },
                b'0'..=b'9' => {
                    // Get the stream length until it meets the colon
                    // TODO handle overflow?
                    let length = self.read_i64_before((unknown - b'0') as i64, b':');
                    self.current_token = Some(Token::StreamLength(length));

                    Token::StreamLength(length)
                }
                b':' => match &self.current_token {
                    Some(Token::StreamLength(_)) => {
                        self.current_token = Some(Token::StreamColon);

                        Token::StreamColon
                    }
                    _ => panic!("Token::StreamColon should be after Token::StreamLength"),
                },
                _ => panic!("Invalid token: {}, position: {}", unknown, &self.position),
            },
            None => Token::EOF,
        }
    }

    fn look_ahead(&mut self) -> Token {
        if let Some(token) = &self.cached_token {
            return token.clone();
        }

        let next_token = self.next_token();
        self.cached_token = Some(next_token);

        next_token
    }
}

pub fn parse<T>(stream: &mut T) -> BNode
where
    T: Iterator<Item = u8>,
{
    let (node, _) = parse_internal(Lexer::new(stream));

    node
}

fn parse_internal<'a, T>(mut lexer: Lexer<'a, T>) -> (BNode, Lexer<'a, T>)
where
    T: Iterator<Item = u8>,
{
    match lexer.look_ahead() {
        Token::IntStart => {
            let (ivalue, _lexer) = parse_int(lexer);

            (BNode::Int(ivalue), _lexer)
        }
        Token::StreamLength(_) => {
            let (stream, _lexer) = parse_stream(lexer);

            (BNode::Stream(stream), _lexer)
        }
        Token::ListStart => {
            let (list, _lexer) = parse_list(lexer);

            (BNode::List(list), _lexer)
        }
        Token::DictStart => {
            let (dict, _lexer) = parse_dict(lexer);

            (BNode::Dict(dict), _lexer)
        }
        _ => panic!("invalid input"),
    }
}

fn parse_int<'a, T>(mut lexer: Lexer<'a, T>) -> (i64, Lexer<'a, T>)
where
    T: Iterator<Item = u8>,
{
    assert_eq!(Token::IntStart, lexer.next_token());

    let value = lexer.read_i64_before(0, b'e');

    assert_eq!(Token::IntEnd, lexer.next_token());

    (value, lexer)
}

fn parse_stream<'a, T>(mut lexer: Lexer<'a, T>) -> (Vec<u8>, Lexer<'a, T>)
where
    T: Iterator<Item = u8>,
{
    let next_token = lexer.next_token();
    match next_token {
        Token::StreamLength(len) => {
            assert_eq!(Token::StreamColon, lexer.next_token());
            let stream = lexer.read_nbytes(len as usize);

            (stream, lexer)
        }
        _ => panic!("invalid input"),
    }
}

fn parse_list<'a, T>(mut lexer: Lexer<'a, T>) -> (BList, Lexer<'a, T>)
where
    T: Iterator<Item = u8>,
{
    assert_eq!(Token::ListStart, lexer.next_token());
    let mut list = vec![];

    loop {
        match lexer.look_ahead() {
            Token::IntStart => {
                let (ivalue, _lexer) = parse_int(lexer);
                list.push(BNode::Int(ivalue));

                lexer = _lexer;
            }
            Token::StreamLength(_) => {
                let (istream, _lexer) = parse_stream(lexer);
                list.push(BNode::Stream(istream));

                lexer = _lexer;
            }
            Token::ListStart => {
                let (ilist, _lexer) = parse_list(lexer);
                list.push(BNode::List(ilist));

                lexer = _lexer;
            }
            Token::DictStart => {
                let (idict, _lexer) = parse_dict(lexer);
                list.push(BNode::Dict(idict));

                lexer = _lexer;
            }
            Token::ListEnd => {
                lexer.next_token();
                return (list, lexer);
            }
            Token::EOF => {
                return (list, lexer);
            }
            _ => panic!("invalid list"),
        }
    }
}

fn parse_dict<'a, T>(mut lexer: Lexer<'a, T>) -> (BDict, Lexer<'a, T>)
where
    T: Iterator<Item = u8>,
{
    assert_eq!(Token::DictStart, lexer.next_token());
    let mut dict = BDict::new();
    loop {
        match lexer.look_ahead() {
            Token::StreamLength(_) => {
                let (raw_key, _lexer) = parse_stream(lexer);
                let key = String::from_utf8(raw_key).unwrap();
                let (value, _lexer) = parse_internal(_lexer);

                lexer = _lexer;

                dict.insert(key, value);
            }
            Token::DictEnd => {
                lexer.next_token();
                return (dict, lexer);
            }
            x => panic!("invalid dictionary, {:?}, position: {}", x, &lexer.position),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::new::{BNode, Token};

    use super::{parse, parse_dict, parse_int, parse_list, parse_stream, Lexer};

    #[test]
    fn test_lexer_read_i64_before() {
        let raw = "2147483648e";
        let mut bytes = raw.bytes();
        let mut lexer = Lexer::new(&mut bytes);

        let value = lexer.read_i64_before(0, b'e');
        assert_eq!(2147483648, value);
    }

    #[test]
    fn test_lexer_read_nbytes() {
        let raw = "bencode";

        let mut bytes = raw.bytes();
        let mut lexer = Lexer::new(&mut bytes);

        let raw_bytes = lexer.read_nbytes(3);
        assert_eq!("ben".as_bytes(), &raw_bytes);

        let raw_bytes = lexer.read_nbytes(4);
        assert_eq!("code".as_bytes(), &raw_bytes);
    }

    #[test]
    fn test_lexer_look_ahead() {
        let raw = "i256e";

        let mut bytes = raw.bytes();
        let mut lexer = Lexer::new(&mut bytes);

        assert_eq!(Token::IntStart, lexer.look_ahead());
        assert_eq!(Token::IntStart, lexer.look_ahead());
    }

    #[test]
    fn test_parse_int() {
        let raw = ["i256e", "i-1024e"];
        let expected = [256, -1024];
        let len = raw.len();

        for x in 0..len {
            let str = raw[x];

            let mut bytes = str.bytes();
            let lexer = Lexer::new(&mut bytes);

            let (value, _lexer) = parse_int(lexer);
            assert_eq!(expected[x], value);
        }
    }

    #[test]
    fn test_parse_stream() {
        let raw = "7:bencode";

        let mut bytes = raw.bytes();
        let lexer = Lexer::new(&mut bytes);

        let (stream, _lexer) = parse_stream(lexer);
        assert_eq!("bencode".as_bytes(), &stream);
    }

    #[test]
    fn test_parse_list() {
        let raw = "li256e7:bencodeli256e7:bencodeee";

        let mut bytes = raw.bytes();
        let lexer = Lexer::new(&mut bytes);

        let (list, _lexer) = parse_list(lexer);
        assert_eq!(3, list.len());
    }

    #[test]
    fn test_parse_nested_list() {
        let raw = "ll5:helloe4:spami42ee";

        let mut bytes = raw.bytes();
        let bnode = parse(&mut bytes);

        let mut buf = vec![];
        bnode.marshal(&mut buf);

        assert_eq!(raw.as_bytes(), &buf);
    }

    #[test]
    fn test_parse_dict() {
        let raw = "d3:bar4:spam3:fooi42ee";

        let mut bytes = raw.bytes();
        let lexer = Lexer::new(&mut bytes);

        let (dict, _lexer) = parse_dict(lexer);
        assert_eq!(2, dict.len());

        match dict.get("bar").unwrap() {
            BNode::Stream(stream) => {
                assert_eq!(&stream, &"spam".as_bytes());
            }
            _ => panic!("`bar` should have the value `spam`"),
        }

        match dict.get("foo").unwrap() {
            BNode::Int(iv) => {
                assert_eq!(&42, iv);
            }
            _ => panic!("`foo` should have the value `42`"),
        }
    }

    #[test]
    fn test_parse_nested_dict() {
        let raw = r#"d8:announce41:http://bttracker.debian.org:6969/announce7:comment35:"Debian CD from cdimage.debian.org"13:creation datei1573903810e9:httpseedsl145:https://cdimage.debian.org/cdimage/release/10.2.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-10.2.0-amd64-netinst.iso145:https://cdimage.debian.org/cdimage/archive/10.2.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-10.2.0-amd64-netinst.isoe4:infod6:lengthi351272960e4:name31:debian-10.2.0-amd64-netinst.iso12:piece lengthi262144eee"#;

        let mut bytes = raw.bytes();
        let bnode = parse(&mut bytes);

        let mut buf = Vec::with_capacity(bytes.len());
        bnode.marshal(&mut buf);

        assert_eq!(&raw.as_bytes(), &buf);
    }
}
