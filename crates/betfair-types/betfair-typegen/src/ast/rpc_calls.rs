pub(crate) use super::*;

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct RpcCall {
    pub(crate) name: Name,
    pub(crate) params: Vec<Param>,
    pub(crate) returns: Returns,
    pub(crate) exception: Option<Exception>,
    pub(crate) description: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct Param {
    pub(crate) name: Name,
    pub(crate) data_type: String,
    pub(crate) mandatory: bool,
    pub(crate) description: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct Returns {
    pub(crate) data_type: String,
    pub(crate) description: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct Exception {
    pub(crate) data_type: String,
    pub(crate) description: Vec<Comment>,
}
