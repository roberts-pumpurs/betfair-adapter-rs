use proc_macro2::TokenStream;
use quote::quote;

use super::GenV1GeneratorStrategy;
use super::injector::CodeInjector;

impl<T: CodeInjector> GenV1GeneratorStrategy<T> {
    pub(crate) fn generate_transport_layer(&self) -> TokenStream {
        quote! {
            pub trait BetfairRpcRequest {
                type Res;
                type Error;

                fn method() -> &'static str;
            }

            use std::fmt;

            use serde::de::{self, Visitor};
            use serde::{Deserialize, Deserializer};
            use std::collections::HashMap;
            use std::hash::Hash;

            use crate::numeric::{F64Ord, NumericOrdPrimitive};

            // Define a custom visitor struct to handle different types
            struct DecimalOptionVisitor;

            impl<'de> Visitor<'de> for DecimalOptionVisitor {
                type Value = Option<NumericOrdPrimitive>;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a string or a floating point number")
                }

                fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Some(F64Ord::new(value)))
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    match value {
                        "NaN" => Ok(None),
                        _ => {
                            let parsed: f64 = value.parse().map_err(E::custom)?;
                            Ok(Some(F64Ord::new(parsed)))
                        }
                    }
                }
            }

            fn deserialize_decimal_option<'de, D>(deserializer: D) -> Result<Option<NumericOrdPrimitive>, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(DecimalOptionVisitor)
            }

            /// For fields of type `Option<HashMap<K, V>>`
            ///
            /// If any of the map values are null, they are ignored and not added
            /// to the resulting HashMap.
            pub fn deserialize_map_skip_null<'de, D, K, V>(
                deserializer: D,
            ) -> Result<Option<HashMap<K, V>>, D::Error>
            where
                D: Deserializer<'de>,
                K: Deserialize<'de> + Eq + Hash,
                V: Deserialize<'de>,
            {
                let opt_map: Option<HashMap<K, Option<V>>> = Option::deserialize(deserializer)?;
                Ok(opt_map.map(|map| {
                    map.into_iter()
                        .filter_map(|(k, v)| v.map(|vv| (k, vv))) // drop entries where value == null
                        .collect()
                }))
            }

            /// For fields of type `HashMap<K, V>`
            ///
            /// Behaves the same as [`deserialize_map_skip_null`] except
            /// that if the overall value is null then an empty HashMap is returned.
            pub fn deserialize_map_skip_null_default_empty<'de, D, K, V>(
                deserializer: D,
            ) -> Result<HashMap<K, V>, D::Error>
            where
                D: Deserializer<'de>,
                K: Deserialize<'de> + Eq + Hash,
                V: Deserialize<'de>,
            {
                deserialize_map_skip_null(deserializer).map(|opt| opt.unwrap_or_default())
            }
        }
    }
}
