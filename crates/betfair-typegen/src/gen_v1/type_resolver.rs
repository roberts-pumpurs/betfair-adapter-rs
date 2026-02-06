use heck::ToPascalCase as _;

use crate::aping_ast::types::DataTypeParameter;
extern crate alloc;
use alloc::borrow::ToOwned as _;

use eyre::{Result, WrapErr as _};

#[derive(Debug)]
pub(crate) struct TypeResolverV1;

pub(crate) struct ResolvedTypeWithMetadata {
    pub resolved_type: syn::Type,
    pub plural: TypePlural,
}

pub(crate) enum TypePlural {
    Singular(String),
    List(String),
    Set(String),
    Map { key: String, value: String },
}

impl TypeResolverV1 {
    pub(crate) const fn new() -> Self {
        Self
    }

    pub(crate) fn resolve_type(&self, data_type: &DataTypeParameter) -> Result<syn::Type> {
        self.resolve_type_with_metadata(data_type)
            .map(|result| result.resolved_type)
    }

    pub(crate) fn resolve_type_with_metadata(
        &self,
        data_type: &DataTypeParameter,
    ) -> Result<ResolvedTypeWithMetadata> {
        fn transform_to_rust_types(input: &str) -> String {
            // TODO make this a configurable thing
            match input {
                "string" => "std::sync::Arc<String>".to_owned(),
                "int" | "i32" => "i32".to_owned(),
                "i64" => "i64".to_owned(),
                "double" | "float" => "crate::numeric::F64Ord".to_owned(),
                "dateTime" => "DateTime<Utc>".to_owned(),
                "boolean" | "bool" => "bool".to_owned(),
                "CustomerRef" => "crate::customer_ref::CustomerRef".to_owned(),
                "CustomerStrategyRef" => {
                    "crate::customer_strategy_ref::CustomerStrategyRef".to_owned()
                }
                "CustomerOrderRef" => "crate::customer_order_ref::CustomerOrderRef".to_owned(),
                "Price" => "crate::price::Price".to_owned(),
                "Size" => "crate::size::Size".to_owned(),
                _ => input.to_pascal_case(),
            }
        }

        let plural = self.manage_list(data_type.as_str());

        let resolved_type = match plural {
            TypePlural::Singular(ref value) => {
                let value = transform_to_rust_types(value);
                syn::parse_str(&value).wrap_err_with(|| format!("Failed to parse type: {value}"))?
            }
            TypePlural::List(ref value) | TypePlural::Set(ref value) => {
                let value = transform_to_rust_types(value);
                let value = format!("Vec<{value}>");
                syn::parse_str(&value).wrap_err_with(|| format!("Failed to parse type: {value}"))?
            }
            TypePlural::Map { ref key, ref value } => {
                let key = transform_to_rust_types(key);
                let value = transform_to_rust_types(value);
                let value = format!("std::collections::HashMap<{key}, {value}>");
                syn::parse_str(&value).wrap_err_with(|| format!("Failed to parse type: {value}"))?
            }
        };

        Ok(ResolvedTypeWithMetadata {
            resolved_type,
            plural,
        })
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
            let inner = item.replace("map(", "").replace(')', "");
            let mut parts = inner.split(',').map(str::trim);
            let key = parts.next().unwrap_or("").to_owned();
            let value = parts.next().unwrap_or("").to_owned();
            TypePlural::Map { key, value }
        } else {
            TypePlural::Singular(item.to_owned())
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
pub(crate) mod tests {
    use proptest::prelude::*;

    use super::*;

    prop_compose! {
        pub(crate) fn valid_data_types()(
            is_list in any::<bool>(),
            data_type in proptest::sample::select(vec!["string", "int", "double", "dateTime", "boolean", "float", "AccountAPINGException", "DeveloperApp"])) -> DataTypeParameter {
            if is_list {
                DataTypeParameter::new(format!("list({data_type})"))
            } else {
                DataTypeParameter::new(data_type.to_owned())
            }
        }
    }

    proptest! {
        #[test]
        fn test_resolve_type(data_type in valid_data_types()) {
            let type_resolver = TypeResolverV1::new();
            type_resolver.resolve_type(&data_type).unwrap();
        }
    }
}
