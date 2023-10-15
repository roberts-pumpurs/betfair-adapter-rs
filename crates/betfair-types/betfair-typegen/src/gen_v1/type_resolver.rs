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
                "string" => "String".to_string(),
                "int" => "i32".to_string(),
                "i32" => "i32".to_string(),
                "i64" => "i64".to_string(),
                "double" => "rust_decimal::Decimal".to_string(),
                "dateTime" => "DateTime<Utc>".to_string(),
                "boolean" => "bool".to_string(),
                "bool" => "bool".to_string(),
                "float" => "rust_decimal::Decimal".to_string(),
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
                let value = format!("Vec<{}>", value);
                syn::parse_str(&value).unwrap()
            }
            TypePlural::Set(value) => {
                let value = transform_to_rust_types(&value);
                let value = format!("Vec<{}>", value);
                syn::parse_str(&value).unwrap()
            }
            TypePlural::Map { key, value } => {
                let key = transform_to_rust_types(&key);
                let value = transform_to_rust_types(&value);
                let value = format!("std::collections::HashMap<{}, {}>", key, value);
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
            let mut item = item.split(',').map(|x| x.to_string());
            let key = item.next().unwrap();
            let value = item.next().unwrap();
            TypePlural::Map { key, value }
        } else {
            TypePlural::Singular(item.to_string())
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
