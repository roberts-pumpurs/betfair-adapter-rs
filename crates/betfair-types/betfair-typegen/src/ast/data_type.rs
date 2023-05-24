pub(crate) use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DataType {
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
    pub(crate) data_type: String,
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
    pub(crate) data_type: String,
    pub(crate) description: Vec<Comment>,
}
