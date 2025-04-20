use proc_macro2::TokenStream;
use quote::quote;

use super::GenV1GeneratorStrategy;
use super::injector::CodeInjector;

impl<T: CodeInjector> GenV1GeneratorStrategy<T> {
    pub(crate) fn generate_transport_layer(&self) -> TokenStream {
        quote! {
            use std::future::Future;
            use serde::{Serialize, de::DeserializeOwned};

            pub trait BetfairRpcRequest {
                type Res;
                type Error;

                fn method() -> &'static str;
            }

            use std::fmt;
            use std::str::FromStr;

            use rust_decimal::{Decimal, prelude::FromPrimitive};
            use serde::de::{self, Visitor};
            use serde::{Deserialize, Deserializer};

            // Define a custom visitor struct to handle different types
            struct DecimalOptionVisitor;

            impl<'de> Visitor<'de> for DecimalOptionVisitor {
                type Value = Option<Decimal>;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a string or a floating point number")
                }

                fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Some(
                        Decimal::from_f64(value).ok_or_else(|| E::custom("Invalid float value"))?,
                    ))
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    match value {
                        "NaN" => Ok(None),
                        _ => Decimal::from_str(value).map(Some).map_err(E::custom),
                    }
                }
            }

            fn deserialize_decimal_option<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(DecimalOptionVisitor)
            }

        }
    }
}
