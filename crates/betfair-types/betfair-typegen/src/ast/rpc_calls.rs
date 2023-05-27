use super::types::DataTypeParameter;
use super::{Comment, Name};

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct RpcCall {
    pub(crate) name: Name,
    pub(crate) params: Vec<Param>,
    pub(crate) returns: Returns,
    pub(crate) exception: Option<Exception>,
    pub(crate) description: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct Param {
    pub(crate) name: Name,
    pub(crate) data_type: DataTypeParameter,
    pub(crate) mandatory: bool,
    pub(crate) description: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct Returns {
    pub(crate) data_type: DataTypeParameter,
    pub(crate) description: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct Exception {
    pub(crate) data_type: DataTypeParameter,
    pub(crate) description: Vec<Comment>,
}
