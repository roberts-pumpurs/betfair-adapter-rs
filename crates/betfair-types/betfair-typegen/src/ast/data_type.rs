use super::types::DataTypeParameter;
use super::{Comment, Name};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DataType {
    pub(crate) name: Name,
    pub(crate) variant: DataTypeVariant,
    pub(crate) description: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DataTypeVariant {
    EnumValue(EnumValue),
    StructValue(StructValue),
    TypeAlias(TypeAlias),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EnumValue {
    pub(crate) name: Name,
    pub(crate) valid_values: Vec<ValidEnumValue>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct ValidEnumValue {
    pub(crate) id: String,
    pub(crate) name: Name,
    pub(crate) description: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct TypeAlias {
    pub(crate) name: Name,
    pub(crate) data_type: DataTypeParameter,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct StructValue {
    pub(crate) name: Name,
    pub(crate) fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct StructField {
    pub(crate) name: Name,
    pub(crate) mandatory: bool,
    pub(crate) data_type: DataTypeParameter,
    pub(crate) description: Vec<Comment>,
}
