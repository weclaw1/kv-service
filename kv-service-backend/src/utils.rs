use std::collections::BTreeMap;

pub fn prost_to_serde_json(value: prost_types::Value) -> serde_json::Value {
    use prost_types::value::Kind::*;
    use serde_json::Value::*;

    match value.kind {
        Some(NullValue(_)) => Null,
        Some(NumberValue(n)) => Number(serde_json::Number::from_f64(n).unwrap()),
        Some(StringValue(s)) => String(s),
        Some(BoolValue(b)) => Bool(b),
        Some(StructValue(s)) => {
            let mut map = serde_json::Map::new();
            for (k, v) in s.fields {
                map.insert(k, prost_to_serde_json(v));
            }
            Object(map)
        }
        Some(ListValue(l)) => {
            let vec = l.values.into_iter().map(prost_to_serde_json).collect();
            Array(vec)
        }
        None => Null,
    }
}

pub fn serde_json_to_prost(value: serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind::*;
    use serde_json::Value::*;

    let kind = match value {
        Null => NullValue(0),
        Number(n) => NumberValue(n.as_f64().unwrap()),
        String(s) => StringValue(s),
        Bool(b) => BoolValue(b),
        Object(map) => {
            let mut fields = BTreeMap::new();
            for (k, v) in map {
                fields.insert(k, serde_json_to_prost(v));
            }
            StructValue(prost_types::Struct { fields })
        }
        Array(vec) => {
            let values = vec.into_iter().map(serde_json_to_prost).collect();
            ListValue(prost_types::ListValue { values })
        }
    };
    prost_types::Value { kind: Some(kind) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prost_to_serde_json() {
        let value = prost_types::Value {
            kind: Some(prost_types::value::Kind::StringValue("test".to_string())),
        };
        let result = prost_to_serde_json(value);
        assert_eq!(result, serde_json::Value::String("test".to_string()));
    }

    #[test]
    fn test_serde_json_to_prost() {
        let value = serde_json::Value::String("test".to_string());
        let result = serde_json_to_prost(value);
        assert_eq!(
            result,
            prost_types::Value {
                kind: Some(prost_types::value::Kind::StringValue("test".to_string())),
            }
        );
    }

    #[test]
    fn test_prost_to_serde_json_null() {
        let value = prost_types::Value {
            kind: Some(prost_types::value::Kind::NullValue(0)),
        };
        let result = prost_to_serde_json(value);
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_serde_json_to_prost_null() {
        let value = serde_json::Value::Null;
        let result = serde_json_to_prost(value);
        assert_eq!(
            result,
            prost_types::Value {
                kind: Some(prost_types::value::Kind::NullValue(0)),
            }
        );
    }

    #[test]
    fn test_prost_to_serde_json_number() {
        let value = prost_types::Value {
            kind: Some(prost_types::value::Kind::NumberValue(1.0)),
        };
        let result = prost_to_serde_json(value);
        assert_eq!(
            result,
            serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap())
        );
    }

    #[test]
    fn test_serde_json_to_prost_number() {
        let value = serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap());
        let result = serde_json_to_prost(value);
        assert_eq!(
            result,
            prost_types::Value {
                kind: Some(prost_types::value::Kind::NumberValue(1.0)),
            }
        );
    }

    #[test]
    fn test_prost_to_serde_json_bool() {
        let value = prost_types::Value {
            kind: Some(prost_types::value::Kind::BoolValue(true)),
        };
        let result = prost_to_serde_json(value);
        assert_eq!(result, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_serde_json_to_prost_bool() {
        let value = serde_json::Value::Bool(true);
        let result = serde_json_to_prost(value);
        assert_eq!(
            result,
            prost_types::Value {
                kind: Some(prost_types::value::Kind::BoolValue(true)),
            }
        );
    }

    #[test]
    fn test_prost_to_serde_json_struct() {
        let value = prost_types::Value {
            kind: Some(prost_types::value::Kind::StructValue(prost_types::Struct {
                fields: vec![(
                    "test".to_string(),
                    prost_types::Value {
                        kind: Some(prost_types::value::Kind::StringValue("test".to_string())),
                    },
                )]
                .into_iter()
                .collect(),
            })),
        };
        let result = prost_to_serde_json(value);
        assert_eq!(
            result,
            serde_json::json!({
                "test": "test"
            })
        );
    }

    #[test]
    fn test_serde_json_to_prost_struct() {
        let value = serde_json::json!({
            "test": "test"
        });
        let result = serde_json_to_prost(value);
        assert_eq!(
            result,
            prost_types::Value {
                kind: Some(prost_types::value::Kind::StructValue(prost_types::Struct {
                    fields: vec![(
                        "test".to_string(),
                        prost_types::Value {
                            kind: Some(prost_types::value::Kind::StringValue("test".to_string())),
                        }
                    )]
                    .into_iter()
                    .collect(),
                })),
            }
        );
    }

    #[test]
    fn test_prost_to_serde_json_list() {
        let value = prost_types::Value {
            kind: Some(prost_types::value::Kind::ListValue(
                prost_types::ListValue {
                    values: vec![prost_types::Value {
                        kind: Some(prost_types::value::Kind::StringValue("test".to_string())),
                    }]
                    .into_iter()
                    .collect(),
                },
            )),
        };
        let result = prost_to_serde_json(value);
        assert_eq!(result, serde_json::json!(["test"]));
    }

    #[test]
    fn test_serde_json_to_prost_list() {
        let value = serde_json::json!(["test"]);
        let result = serde_json_to_prost(value);
        assert_eq!(
            result,
            prost_types::Value {
                kind: Some(prost_types::value::Kind::ListValue(
                    prost_types::ListValue {
                        values: vec![prost_types::Value {
                            kind: Some(prost_types::value::Kind::StringValue("test".to_string())),
                        }]
                        .into_iter()
                        .collect(),
                    }
                )),
            }
        );
    }
}
