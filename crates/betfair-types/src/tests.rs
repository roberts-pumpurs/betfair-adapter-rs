mod f64_deserialization_tests {
    use crate::numeric::F64Ord;
    use serde::Deserialize;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }
    fn is_pos_inf(x: f64) -> bool {
        x.is_infinite() && x.is_sign_positive()
    }
    fn is_neg_inf(x: f64) -> bool {
        x.is_infinite() && x.is_sign_negative()
    }

    mod deserialize_f64_option_tests {

        use super::*;

        #[derive(Deserialize, Debug)]
        struct TestF64Option {
            #[serde(deserialize_with = "crate::types::deserialize_f64_option")]
            v: Option<F64Ord>,
        }

        #[test]
        fn null_is_none() {
            let w: TestF64Option = serde_json::from_str(r#"{ "v": null }"#).unwrap();
            assert!(w.v.is_none());
        }

        #[test]
        fn number_and_string_number() {
            let a: TestF64Option = serde_json::from_str(r#"{ "v": 5.2 }"#).unwrap();
            let b: TestF64Option = serde_json::from_str(r#"{ "v": "5.2" }"#).unwrap();
            assert!(approx_eq(a.v.unwrap().0, 5.2, 1e-12));
            assert!(approx_eq(b.v.unwrap().0, 5.2, 1e-12));
        }

        #[test]
        fn integer_tokens() {
            let a: TestF64Option = serde_json::from_str(r#"{ "v": 42 }"#).unwrap();
            let b: TestF64Option = serde_json::from_str(r#"{ "v": "42" }"#).unwrap();
            assert!(approx_eq(a.v.unwrap().0, 42.0, 1e-12));
            assert!(approx_eq(b.v.unwrap().0, 42.0, 1e-12));
        }

        #[test]
        fn special_values_basic() {
            let nan: TestF64Option = serde_json::from_str(r#"{ "v": "NaN" }"#).unwrap();
            let pinf: TestF64Option = serde_json::from_str(r#"{ "v": "Infinity" }"#).unwrap();
            let ninf: TestF64Option = serde_json::from_str(r#"{ "v": "-Infinity" }"#).unwrap();
            assert!(nan.v.unwrap().as_f64().is_nan());
            assert!(is_pos_inf(pinf.v.unwrap().0));
            assert!(is_neg_inf(ninf.v.unwrap().0));
        }

        #[test]
        fn special_values_aliases_and_case() {
            let a: TestF64Option = serde_json::from_str(r#"{ "v": "inf" }"#).unwrap();
            let b: TestF64Option = serde_json::from_str(r#"{ "v": "-inf" }"#).unwrap();
            let c: TestF64Option = serde_json::from_str(r#"{ "v": "InFiNiTy" }"#).unwrap();
            let d: TestF64Option = serde_json::from_str(r#"{ "v": "  -InFiNiTy  " }"#).unwrap();
            assert!(is_pos_inf(a.v.unwrap().0));
            assert!(is_neg_inf(b.v.unwrap().0));
            assert!(is_pos_inf(c.v.unwrap().0));
            assert!(is_neg_inf(d.v.unwrap().0));
        }

        #[test]
        fn invalid_string_errors() {
            let err = serde_json::from_str::<TestF64Option>(r#"{ "v": "abc" }"#).unwrap_err();
            let msg = err.to_string().to_lowercase();
            assert!(msg.contains("invalid f64"), "unexpected error: {msg}");
        }
    }

    mod deserialize_f64_tests {
        use super::*;

        #[derive(Deserialize, Debug)]
        struct TestF64 {
            #[serde(deserialize_with = "crate::types::deserialize_f64")]
            v: F64Ord,
        }

        #[test]
        fn errors_on_null() {
            let err = serde_json::from_str::<TestF64>(r#"{ "v": null }"#).unwrap_err();
            let msg = err.to_string().to_lowercase();
            assert!(
                msg.contains("invalid type") || msg.contains("null"),
                "unexpected error: {msg}"
            );
        }

        #[test]
        fn numbers_and_special() {
            let a: TestF64 = serde_json::from_str(r#"{ "v": 1 }"#).unwrap();
            let b: TestF64 = serde_json::from_str(r#"{ "v": "1.25" }"#).unwrap();
            let c: TestF64 = serde_json::from_str(r#"{ "v": "nan" }"#).unwrap();
            assert!(approx_eq(a.v.0, 1.0, 1e-12));
            assert!(approx_eq(b.v.0, 1.25, 1e-12));
            assert!(c.v.as_f64().is_nan());
        }

        #[test]
        fn whitespace_string_parses() {
            let a: TestF64 = serde_json::from_str(r#"{ "v": "  10.5  " }"#).unwrap();
            assert!(approx_eq(a.v.0, 10.5, 1e-12));
        }
    }
}
