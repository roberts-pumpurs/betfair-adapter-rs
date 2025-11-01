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

            #[cfg(feature = "decimal-primitives")]
            use rust_decimal::{Decimal, prelude::FromPrimitive};
            #[cfg(feature = "decimal-primitives")]
            use std::str::FromStr;

            #[cfg(not(feature = "decimal-primitives"))]
            use crate::numeric::{F64Ord};

            use crate::numeric::{NumericOrdPrimitive};

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
                    #[cfg(not(feature = "decimal-primitives"))]
                    {
                        Ok(Some(F64Ord::new(value)))
                    }
                    #[cfg(feature = "decimal-primitives")]
                    {
                        Ok(Some(
                            Decimal::from_f64(value).ok_or_else(|| E::custom("Invalid float value"))?,
                        ))
                    }
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    match value {
                        "NaN" => Ok(None),
                        #[cfg(not(feature = "decimal-primitives"))]
                        _ => {
                            let parsed: f64 = value.parse().map_err(E::custom)?;
                            Ok(Some(F64Ord::new(parsed)))
                        }
                        #[cfg(feature = "decimal-primitives")]
                        _ => Decimal::from_str(value).map(Some).map_err(E::custom),
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
