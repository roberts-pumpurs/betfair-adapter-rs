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
            use crate::numeric::{F64Ord};

            fn parse_special_str(s: &str) -> Option<f64> {
                let t = s.trim();
                if t.eq_ignore_ascii_case("nan") {
                    Some(f64::NAN)
                } else if t.eq_ignore_ascii_case("infinity")
                    || t.eq_ignore_ascii_case("+infinity")
                    || t.eq_ignore_ascii_case("inf")
                    || t.eq_ignore_ascii_case("+inf")
                {
                    Some(f64::INFINITY)
                } else if t.eq_ignore_ascii_case("-infinity") || t.eq_ignore_ascii_case("-inf") {
                    Some(f64::NEG_INFINITY)
                } else {
                    None
                }
            }

            fn parse_str_as_f64<E: de::Error>(s: &str) -> Result<f64, E> {
                if let Some(v) = parse_special_str(s) {
                    Ok(v)
                } else {
                    s.trim()
                        .parse::<f64>()
                        .map_err(|e| E::custom(format!("invalid f64 string '{s}': {e}")))
                }
            }

            /// Single visitor that yields Option<f64>.
            /// - null/unit/none -> None
            /// - numbers (u64/i64/f64) -> Some(value)
            /// - strings: "NaN"/"Infinity"/"-Infinity"/"inf"/... or numeric -> Some(value)
            struct F64OptionVisitor;

            impl<'de> Visitor<'de> for F64OptionVisitor {
                type Value = Option<f64>;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("null or a number/numeric string/special string (NaN/Infinity/-Infinity)")
                }

                fn visit_unit<E>(self) -> Result<Self::Value, E> {
                    Ok(None)
                }
                fn visit_none<E>(self) -> Result<Self::Value, E> {
                    Ok(None)
                }
                fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    // Recurse to parse the inner non-null value.
                    d.deserialize_any(F64OptionVisitor)
                }

                fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                    Ok(Some(v))
                }
                fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                    Ok(Some(v as f64))
                }
                fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                    Ok(Some(v as f64))
                }
                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    parse_str_as_f64::<E>(v).map(Some)
                }
                fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    parse_str_as_f64::<E>(v).map(Some)
                }
                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    parse_str_as_f64::<E>(&v).map(Some)
                }
            }

            pub fn deserialize_f64_option<'de, D>(de: D) -> Result<Option<F64Ord>, D::Error>
            where
                D: Deserializer<'de>,
            {
                match de.deserialize_any(F64OptionVisitor)? {
                    Some(v) => Ok(Some(F64Ord(v))),
                    None => Ok(None),
                }
            }

            pub fn deserialize_f64<'de, D>(de: D) -> Result<F64Ord, D::Error>
            where
                D: Deserializer<'de>,
            {
                match de.deserialize_any(F64OptionVisitor)? {
                    Some(v) => Ok(F64Ord(v)),
                    None => Err(de::Error::invalid_type(
                        de::Unexpected::Unit,
                        &"non-null number, numeric string, or special string (NaN/Infinity/-Infinity)",
                    )),
                }
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
