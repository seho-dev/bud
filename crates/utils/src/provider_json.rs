use serde_json::Value;
use shared_types::ProviderValue;

pub fn provider_value_to_json(val: &ProviderValue) -> Value {
  match val {
    ProviderValue::Null => Value::Null,
    ProviderValue::Bool(b) => Value::Bool(*b),
    ProviderValue::Int(i) => Value::Number((*i).into()),
    ProviderValue::Float(f) => serde_json::Number::from_f64(*f).map_or(Value::Null, Value::Number),
    ProviderValue::String(s) => Value::String(s.clone()),
    ProviderValue::Array(arr) => Value::Array(arr.iter().map(provider_value_to_json).collect()),
    ProviderValue::Object(obj) => Value::Object(
      obj
        .iter()
        .map(|(k, v)| (k.clone(), provider_value_to_json(v)))
        .collect(),
    ),
  }
}

pub fn json_to_provider_value(val: &Value) -> ProviderValue {
  match val {
    Value::Null => ProviderValue::Null,
    Value::Bool(b) => ProviderValue::Bool(*b),
    Value::Number(n) => {
      if let Some(i) = n.as_i64() {
        ProviderValue::Int(i)
      } else if let Some(f) = n.as_f64() {
        ProviderValue::Float(f)
      } else {
        ProviderValue::Null
      }
    }
    Value::String(s) => ProviderValue::String(s.clone()),
    Value::Array(arr) => ProviderValue::Array(arr.iter().map(json_to_provider_value).collect()),
    Value::Object(obj) => ProviderValue::Object(
      obj
        .iter()
        .map(|(k, v)| (k.clone(), json_to_provider_value(v)))
        .collect(),
    ),
  }
}

pub fn args_to_json(args: &[ProviderValue]) -> Value {
  Value::Array(args.iter().map(provider_value_to_json).collect())
}
