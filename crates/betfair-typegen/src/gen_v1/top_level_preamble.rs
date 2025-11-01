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
            use serde::{Deserializer};

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

        }
    }
}
