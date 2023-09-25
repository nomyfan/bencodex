use crate::BNode;

impl From<i64> for BNode {
    fn from(value: i64) -> Self {
        BNode::Integer(value)
    }
}

impl From<String> for BNode {
    fn from(value: String) -> Self {
        BNode::Bytes(value.into())
    }
}

impl From<&str> for BNode {
    fn from(value: &str) -> Self {
        BNode::Bytes(value.into())
    }
}

impl From<Vec<u8>> for BNode {
    fn from(value: Vec<u8>) -> Self {
        BNode::Bytes(value)
    }
}

impl From<&[u8]> for BNode {
    fn from(value: &[u8]) -> Self {
        BNode::Bytes(value.into())
    }
}

impl From<Vec<BNode>> for BNode {
    fn from(value: Vec<BNode>) -> Self {
        BNode::List(value)
    }
}

impl From<std::collections::BTreeMap<String, BNode>> for BNode {
    fn from(value: std::collections::BTreeMap<String, BNode>) -> Self {
        BNode::Dict(value)
    }
}
