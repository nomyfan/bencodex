#![allow(semicolon_in_expressions_from_macros)]

use std::{fmt::Display, io::Write};
pub type BList = Vec<BNode>;
pub type BDict = std::collections::BTreeMap<String, BNode>;

#[derive(Debug)]
pub struct Error {
    pub position: i64,
    pub msg: String,
}

macro_rules! throw {
    ($msg:expr, $pos:expr) => {
        return Err(Error {
            msg: $msg.into(),
            position: $pos,
        });
    };
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BNode {
    Integer(i64),
    Bytes(Vec<u8>),
    List(BList),
    Dict(BDict),
}

impl BNode {
    pub fn marshal<W>(&self, buf: &mut W) -> std::io::Result<usize>
    where
        W: Write,
    {
        let mut w = 0;
        match self {
            BNode::Integer(i) => {
                w += buf.write(b"i")?;
                w += buf.write(i.to_string().as_bytes())?;
                w += buf.write(b"e")?;
            }
            BNode::Bytes(s) => {
                w += buf.write(s.len().to_string().as_bytes())?;
                w += buf.write(b":")?;
                w += buf.write(s)?;
            }
            BNode::List(l) => {
                w += buf.write(b"l")?;
                for bn in l {
                    w += bn.marshal(buf)?;
                }
                w += buf.write(b"e")?;
            }
            BNode::Dict(m) => {
                w += buf.write(b"d")?;
                for (k, v) in m {
                    w += buf.write(k.len().to_string().as_bytes())?;
                    w += buf.write(b":")?;
                    w += buf.write(k.as_bytes())?;
                    w += v.marshal(buf)?;
                }
                w += buf.write(b"e")?;
            }
        }

        Ok(w)
    }

    pub fn as_integer(&self) -> std::result::Result<&i64, String> {
        match self {
            BNode::Integer(value) => Ok(value),
            _ => Err("not an integer".into()),
        }
    }

    pub fn as_bytes(&self) -> std::result::Result<&[u8], String> {
        match self {
            BNode::Bytes(bytes) => Ok(bytes),
            _ => Err("not a byte array".into()),
        }
    }

    pub fn as_list(&self) -> std::result::Result<&[BNode], String> {
        match self {
            BNode::List(list) => Ok(list),
            _ => Err("not a list".into()),
        }
    }

    pub fn as_dict(&self) -> std::result::Result<&BDict, String> {
        match self {
            BNode::Dict(dict) => Ok(dict),
            _ => Err("not a dictionary".into()),
        }
    }
}

impl Display for BNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut buf = vec![];

        // TODO: return error
        self.marshal(&mut buf).unwrap();
        write!(f, "{}", std::str::from_utf8(&buf).unwrap()).unwrap();

        Ok(())
    }
}

/// https://en.wikipedia.org/wiki/Bencode
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Token {
    IntegerBegin,
    IntegerEnd,
    ListBegin,
    ListEnd,
    DictBegin,
    DictEnd,
    Length(i64),
    Colon,
    EOF,
}

#[derive(Debug)]
struct Lexer<'a, T>
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
        self.position += 1;
        match self.cached_byte {
            Some(_) => self.cached_byte.take(),
            None => self.stream.next(),
        }
    }

    fn read_i64_before(&mut self, init: i64, symbol: u8) -> Result<(i64, i64)> {
        let mut num = init;
        let mut sign = 1i64;
        let mut read = 0;

        while let Some(x) = self.next_byte() {
            read += 1;

            match x {
                b'0'..=b'9' => {
                    if x == b'0' && sign == -1 && read == 2 {
                        throw!("Negative zero is not permitted", self.position)
                    }

                    if num == 0 && ((sign == 1 && read != 1) || (sign == -1 && read != 2)) {
                        throw!("Leading zero is not permitted", self.position)
                    }

                    num = num * 10 + (x - b'0') as i64
                }
                b'-' => match sign {
                    -1 if read != 1 => {
                        throw!(
                            "`-` can only appear in the head of the integer",
                            self.position
                        )
                    }
                    _ => sign = -1,
                },
                b if b == symbol => {
                    self.cached_byte = Some(symbol);
                    self.position -= 1;
                    return Ok((sign * num, read - 1));
                }
                _ => throw!("invalid integer", self.position),
            }
        }

        throw!("invalid integer", self.position)
    }

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut ret = Vec::with_capacity(len);

        for _ in 0..len {
            match self.next_byte() {
                Some(byte) => ret.push(byte),
                None => {
                    throw!(
                        format!(
                            "bytes's length is expected to be {}, but it's {}.",
                            len,
                            ret.len()
                        ),
                        self.position
                    );
                }
            }
        }

        Ok(ret)
    }

    fn next_token(&mut self) -> Result<Token> {
        if let Some(token) = self.cached_token.take() {
            return Ok(token);
        }

        match self.next_byte() {
            Some(unknown) => match unknown {
                b'i' => {
                    self.current_token = Some(Token::IntegerBegin);
                    self.token_stack.push(Token::IntegerBegin);

                    Ok(Token::IntegerBegin)
                }
                b'l' => {
                    self.current_token = Some(Token::ListBegin);
                    self.token_stack.push(Token::ListBegin);

                    Ok(Token::ListBegin)
                }
                b'd' => {
                    self.current_token = Some(Token::DictBegin);
                    self.token_stack.push(Token::DictBegin);

                    Ok(Token::DictBegin)
                }
                b'e' => match &self.token_stack.pop() {
                    Some(Token::IntegerBegin) => {
                        self.current_token = None;

                        Ok(Token::IntegerEnd)
                    }
                    Some(Token::ListBegin) => {
                        self.current_token = None;

                        Ok(Token::ListEnd)
                    }
                    Some(Token::DictBegin) => {
                        self.current_token = None;

                        Ok(Token::DictEnd)
                    }
                    _ => {
                        throw!(
                            "`e` should be the end of integer, list and dictionary.",
                            self.position
                        )
                    }
                },
                b'0'..=b'9' => {
                    // Get the bytes length until it meets the colon
                    // TODO handle overflow?
                    let (length, _) = self.read_i64_before((unknown - b'0') as i64, b':')?;
                    self.current_token = Some(Token::Length(length));

                    Ok(Token::Length(length))
                }
                b':' => match &self.current_token {
                    Some(Token::Length(_)) => {
                        self.current_token = Some(Token::Colon);

                        Ok(Token::Colon)
                    }
                    _ => throw!("`:` should be after the length of bytes.", self.position),
                },
                _ => throw!(format!("unknown token: {}", unknown), self.position),
            },
            None => Ok(Token::EOF),
        }
    }

    fn look_ahead(&mut self) -> Result<Token> {
        if let Some(token) = &self.cached_token {
            return Ok(*token);
        }

        let next_token = self.next_token()?;
        self.cached_token = Some(next_token);

        Ok(next_token)
    }
}

pub struct Parser<'a, T>
where
    T: Iterator<Item = u8>,
{
    lexer: Lexer<'a, T>,
}

impl<'a, T> Parser<'a, T>
where
    T: Iterator<Item = u8>,
{
    pub fn new(stream: &'a mut T) -> Parser<'a, T> {
        Parser {
            lexer: Lexer::new(stream),
        }
    }

    pub fn parse(&mut self) -> Result<BNode>
    where
        T: Iterator<Item = u8>,
    {
        let node = self.parse_node()?;

        match self.lexer.next_token()? {
            Token::EOF => Ok(node),
            _ => throw!("Expect EOF", self.lexer.position),
        }
    }

    fn parse_node(&mut self) -> Result<BNode>
    where
        T: Iterator<Item = u8>,
    {
        match self.lexer.look_ahead()? {
            Token::IntegerBegin => Ok(BNode::Integer(self.parse_integer()?)),
            Token::Length(_) => Ok(BNode::Bytes(self.parse_bytes()?)),
            Token::ListBegin => Ok(BNode::List(self.parse_list()?)),
            Token::DictBegin => Ok(BNode::Dict(self.parse_dict()?)),
            _ => throw!("invalid input", self.lexer.position),
        }
    }

    fn parse_integer(&mut self) -> Result<i64>
    where
        T: Iterator<Item = u8>,
    {
        assert_eq!(Token::IntegerBegin, self.lexer.next_token()?);

        let (value, read) = self.lexer.read_i64_before(0, b'e')?;

        if read < 1 {
            throw!("Integer cannot be empty", self.lexer.position)
        }

        assert_eq!(Token::IntegerEnd, self.lexer.next_token()?);

        Ok(value)
    }

    fn parse_bytes(&mut self) -> Result<Vec<u8>>
    where
        T: Iterator<Item = u8>,
    {
        let next_token = self.lexer.next_token()?;
        match next_token {
            Token::Length(len) => {
                assert_eq!(Token::Colon, self.lexer.next_token()?);
                Ok(self.lexer.read_bytes(len as usize)?)
            }
            _ => throw!("invalid input", self.lexer.position),
        }
    }

    fn parse_list(&mut self) -> Result<BList>
    where
        T: Iterator<Item = u8>,
    {
        assert_eq!(Token::ListBegin, self.lexer.next_token()?);
        let mut list = vec![];

        loop {
            match self.lexer.look_ahead()? {
                Token::IntegerBegin => {
                    list.push(BNode::Integer(self.parse_integer()?));
                }
                Token::Length(_) => {
                    list.push(BNode::Bytes(self.parse_bytes()?));
                }
                Token::ListBegin => {
                    list.push(BNode::List(self.parse_list()?));
                }
                Token::DictBegin => {
                    list.push(BNode::Dict(self.parse_dict()?));
                }
                Token::ListEnd => {
                    self.lexer.next_token()?;
                    return Ok(list);
                }
                _ => {
                    throw!("invalid list", self.lexer.position);
                }
            }
        }
    }

    fn parse_dict(&mut self) -> Result<BDict>
    where
        T: Iterator<Item = u8>,
    {
        assert_eq!(Token::DictBegin, self.lexer.next_token()?);
        let mut dict = BDict::new();
        loop {
            match self.lexer.look_ahead()? {
                Token::Length(_) => {
                    let raw_key = self.parse_bytes()?;
                    let key = String::from_utf8(raw_key).unwrap();
                    let value = self.parse_node()?;
                    dict.insert(key, value);
                }
                Token::DictEnd => {
                    self.lexer.next_token()?;
                    return Ok(dict);
                }
                _ => throw!("invalid dictionary", self.lexer.position),
            }
        }
    }
}

pub fn parse<T>(stream: &mut T) -> Result<BNode>
where
    T: Iterator<Item = u8>,
{
    let mut parser = Parser::new(stream);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::{BNode, Lexer, Parser, Token};

    #[test]
    fn test_lexer_read_i64_before() {
        let raws = ["2147483648e", "0e"];
        let ret = [2147483648, 0];

        for i in 0..raws.len() {
            let raw = raws[i];
            let mut bytes = raw.bytes();
            let mut lexer = Lexer::new(&mut bytes);

            let (value, _) = lexer.read_i64_before(0, b'e').unwrap();
            assert_eq!(ret[i], value);
        }
    }

    #[test]
    fn test_lexer_read_negative_zero() {
        let raw = "-0e";

        let mut bytes = raw.bytes();
        let mut lexer = Lexer::new(&mut bytes);

        let _ = lexer
            .read_i64_before(0, b'e')
            .expect_err("Negative zero is not permitted");
    }

    #[test]
    fn test_lexer_no_leading_zero() {
        let raws = ["00e", "01e"];

        for raw in raws.iter() {
            let mut bytes = raw.bytes();
            let mut lexer = Lexer::new(&mut bytes);

            let _ = lexer
                .read_i64_before(0, b'e')
                .expect_err("Leading zero is not permitted");
        }
    }

    #[test]
    fn test_lexer_read_bytes() {
        let mut bytes = "bencode".bytes();
        let mut lexer = Lexer::new(&mut bytes);

        let raw_bytes = lexer.read_bytes(3).unwrap();
        assert_eq!("ben".as_bytes(), &raw_bytes);

        let raw_bytes = lexer.read_bytes(4).unwrap();
        assert_eq!("code".as_bytes(), &raw_bytes);
    }

    #[test]
    fn test_lexer_position_read_bytes() {
        let mut bytes = "bencode".bytes();
        let mut lexer = Lexer::new(&mut bytes);

        let _ = lexer.read_bytes(3).unwrap();
        assert_eq!(2, lexer.position);

        let _ = lexer.read_bytes(4).unwrap();
        assert_eq!(6, lexer.position);
    }

    #[test]
    fn test_lexer_position_cache_token() {
        let mut bytes = "i56e".bytes();
        let mut lexer = Lexer::new(&mut bytes);

        let _ = lexer.look_ahead().unwrap();
        assert_eq!(0, lexer.position);

        let _ = lexer.look_ahead().unwrap();
        assert_eq!(0, lexer.position);
    }

    #[test]
    fn test_lexer_position_read_i64_before() {
        let mut bytes = "7:bencode".bytes();
        let mut lexer = Lexer::new(&mut bytes);

        lexer.read_i64_before(0, b':').unwrap();
        assert_eq!(0, lexer.position);
        lexer.read_bytes(1).unwrap();
        assert_eq!(1, lexer.position);
    }

    #[test]
    fn test_lexer_position_error() {
        let mut bytes = "i-2-0e".bytes();
        let mut parser = Parser::new(&mut bytes);

        assert_eq!(3, parser.parse_integer().unwrap_err().position)
    }

    #[test]
    fn test_lexer_look_ahead() {
        let mut bytes = "i256e".bytes();
        let mut lexer = Lexer::new(&mut bytes);

        assert_eq!(Token::IntegerBegin, lexer.look_ahead().unwrap());
        assert_eq!(Token::IntegerBegin, lexer.look_ahead().unwrap());
    }

    #[test]
    fn test_parse_integer() {
        let raw = ["i256e", "i-1024e"];
        let expected = [256, -1024];
        for (raw, expected) in raw.iter().zip(expected) {
            let mut bytes = raw.bytes();
            let mut parser = Parser::new(&mut bytes);

            let value = parser.parse_integer().unwrap();
            assert_eq!(expected, value);
        }
    }

    #[test]
    fn test_parse_integer_failed() {
        let cases = ["i2522", "ie", "i", "i-12-3e", "i13ee"];
        for (i, _) in cases.iter().enumerate() {
            let x = cases[i];
            let mut bytes = x.bytes();
            let mut parser = Parser::new(&mut bytes);
            if parser.parse().is_ok() {
                panic!("{}-th should fail", i);
            }
        }
    }

    #[test]
    fn test_parse_bytes() {
        let mut bytes = "7:bencode".bytes();
        let mut parser = Parser::new(&mut bytes);

        let bytes = parser.parse_bytes().unwrap();
        assert_eq!("bencode".as_bytes(), &bytes);
    }

    #[test]
    fn test_parse_bytes_failed() {
        let cases = ["5:hello2", "5:halo", "521"];
        for (i, _) in cases.iter().enumerate() {
            let mut bytes = cases[i].bytes();
            let mut parser = Parser::new(&mut bytes);
            if parser.parse().is_ok() {
                panic!("{}-th should fail", i);
            }
        }
    }

    #[test]
    fn test_parse_list() {
        let cases = ["li256e7:bencodeli256e7:bencodeee", "l4:spami42ee", "le"];
        for (i, _) in cases.iter().enumerate() {
            let mut bytes = cases[i].bytes();
            let mut parser = Parser::new(&mut bytes);
            match parser.parse() {
                Ok(node) => {
                    let mut buf = vec![];
                    let _ = node.marshal(&mut buf);
                    assert_eq!(cases[i].as_bytes(), &buf)
                }
                Err(e) => std::panic::panic_any(e),
            }
        }
    }

    #[test]
    fn test_parse_list_failed() {
        let cases = ["l4:halo"];
        for (i, _) in cases.iter().enumerate() {
            let mut bytes = cases[i].bytes();
            let mut parser = Parser::new(&mut bytes);
            if parser.parse().is_ok() {
                panic!("{}-th should fail", i);
            }
        }
    }

    #[test]
    fn test_parse_nested_list() {
        let raw = "ll5:helloe4:spami42ee";
        let mut bytes = raw.bytes();
        let mut parser = Parser::new(&mut bytes);
        let bnode = parser.parse().unwrap();

        let mut buf = vec![];
        let _ = bnode.marshal(&mut buf).unwrap();

        assert_eq!(raw.as_bytes(), &buf);
    }

    #[test]
    fn test_parse_dict() {
        let raw = "d3:bar4:spam3:fooi42ee";

        let mut bytes = raw.bytes();
        let mut parser = Parser::new(&mut bytes);

        let dict = parser.parse_dict().unwrap();
        assert_eq!(2, dict.len());

        match dict.get("bar").unwrap() {
            BNode::Bytes(bytes) => {
                assert_eq!(&bytes, &"spam".as_bytes());
            }
            _ => panic!("`bar` should have the value `spam`"),
        }

        match dict.get("foo").unwrap() {
            BNode::Integer(iv) => {
                assert_eq!(&42, iv);
            }
            _ => panic!("`foo` should have the value `42`"),
        }
    }

    #[test]
    fn test_parse_dict_failed() {
        let cases = ["d4:haloi23e", "di23e4:haloe"];
        for x in &cases {
            let mut bytes = x.bytes();
            let mut parser = Parser::new(&mut bytes);
            if parser.parse().is_ok() {
                panic!("Should fail");
            }
        }
    }

    #[test]
    fn test_parse_nested_dict() {
        let raw = r#"d8:announce41:http://bttracker.debian.org:6969/announce7:comment35:"Debian CD from cdimage.debian.org"13:creation datei1573903810e9:httpseedsl145:https://cdimage.debian.org/cdimage/release/10.2.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-10.2.0-amd64-netinst.iso145:https://cdimage.debian.org/cdimage/archive/10.2.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-10.2.0-amd64-netinst.isoe4:infod6:lengthi351272960e4:name31:debian-10.2.0-amd64-netinst.iso12:piece lengthi262144eee"#;

        let mut bytes = raw.bytes();
        let mut parser = Parser::new(&mut bytes);
        let bnode = parser.parse().unwrap();

        let mut buf = Vec::with_capacity(bytes.len());
        let _ = bnode.marshal(&mut buf);

        assert_eq!(&raw.as_bytes(), &buf);
    }
}
