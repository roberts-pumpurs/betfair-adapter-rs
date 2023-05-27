use heck::ToPascalCase;

use crate::ast::types::DataTypeParameter;

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
                "double" => "FixedNumber".to_string(),
                "dateTime" => "DateTime<Utc>".to_string(),
                "boolean" => "bool".to_string(),
                "float" => "FixedNumber".to_string(),
                _ => input.to_pascal_case(),
            }
        }
        match self.manage_list(data_type.as_str()) {
            TypePlural::Singular(value) => {
                let value = transform_to_rust_types(&value);
                syn::parse_str(&value).unwrap()
            }
            TypePlural::Plural(value) => {
                let value = transform_to_rust_types(&value);
                let value = format!("Vec<{}>", value);
                syn::parse_str(&value).unwrap()
            }
        }
    }

    fn manage_list(&self, item: &str) -> TypePlural {
        if item.contains("list(") {
            let item = item.replace("list(", "");
            let item = item.replace(')', "");
            TypePlural::Plural(item)
        } else {
            TypePlural::Singular(item.to_string())
        }
    }
}

enum TypePlural {
    Singular(String),
    Plural(String),
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
