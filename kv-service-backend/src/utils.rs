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
