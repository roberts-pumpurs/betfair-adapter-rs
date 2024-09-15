use heck::ToPascalCase;

use crate::aping_ast::types::DataTypeParameter;

#[derive(Debug)]
pub(crate) struct TypeResolverV1;

impl TypeResolverV1 {
    pub(crate) const fn new() -> Self {
        Self
    }

    pub(crate) fn resolve_type(&self, data_type: &DataTypeParameter) -> syn::Type {
        fn transform_to_rust_types(input: &str) -> String {
            // TODO make this a configurable thing
            match input {
                "string" => "String".to_owned(),
                "int" => "i32".to_owned(),
                "i32" => "i32".to_owned(),
                "i64" => "i64".to_owned(),
                "double" => "rust_decimal::Decimal".to_owned(),
                "dateTime" => "DateTime<Utc>".to_owned(),
                "boolean" => "bool".to_owned(),
                "bool" => "bool".to_owned(),
                "CustomerRef" => "crate::customer_ref::CustomerRef".to_owned(),
                "CustomerStrategyRef" => {
                    "crate::customer_strategy_ref::CustomerStrategyRef".to_owned()
                }
                "CustomerOrderRef" => "crate::customer_order_ref::CustomerOrderRef".to_owned(),
                "Price" => "crate::price::Price".to_owned(),
                "Size" => "crate::size::Size".to_owned(),
                "float" => "rust_decimal::Decimal".to_owned(),
                _ => input.to_pascal_case(),
            }
        }
        match self.manage_list(data_type.as_str()) {
            TypePlural::Singular(value) => {
                let value = transform_to_rust_types(&value);
                syn::parse_str(&value).unwrap()
            }
            TypePlural::List(value) => {
                let value = transform_to_rust_types(&value);
                let value = format!("Vec<{value}>");
                syn::parse_str(&value).unwrap()
            }
            TypePlural::Set(value) => {
                let value = transform_to_rust_types(&value);
                let value = format!("Vec<{value}>");
                syn::parse_str(&value).unwrap()
            }
            TypePlural::Map { key, value } => {
                let key = transform_to_rust_types(&key);
                let value = transform_to_rust_types(&value);
                let value = format!("std::collections::HashMap<{key}, {value}>");
                syn::parse_str(&value).unwrap()
            }
        }
    }

    fn manage_list(&self, item: &str) -> TypePlural {
        if item.contains("list(") {
            let item = item.replace("list(", "");
            let item = item.replace(')', "");
            TypePlural::List(item)
        } else if item.contains("set(") {
            let item = item.replace("set(", "");
            let item = item.replace(')', "");
            TypePlural::Set(item)
        } else if item.contains("map(") {
            let item = item.replace("map(", "");
            let item = item.replace(')', "");
            let mut item = item.split(',').map(std::borrow::ToOwned::to_owned);
            let key = item.next().unwrap();
            let value = item.next().unwrap();
            TypePlural::Map { key, value }
        } else {
            TypePlural::Singular(item.to_owned())
        }
    }
}

enum TypePlural {
    Singular(String),
    List(String),
    Set(String),
    Map { key: String, value: String },
}

#[cfg(test)]
pub(crate) mod tests {
    use proptest::prelude::*;

    use super::*;

    prop_compose! {
        pub(crate) fn valid_data_types()(
            is_list in any::<bool>(),
            data_type in proptest::sample::select(vec!["string", "int", "double", "dateTime", "boolean", "float", "AccountAPINGException", "DeveloperApp"])) -> DataTypeParameter {
            if is_list {
                DataTypeParameter::new(format!("list({})", data_type))
            } else {
                DataTypeParameter::new(data_type.to_string())
            }
        }
    }

    proptest! {
        #[test]
        fn test_resolve_type(data_type in valid_data_types()) {
            let type_resolver = TypeResolverV1::new();
            type_resolver.resolve_type(&data_type);
        }
    }
}
