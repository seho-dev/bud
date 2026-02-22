use bud_plugin_sdk::{register, Plugin};

struct SumPlugin;

impl Plugin for SumPlugin {
  fn on_invoke(function: &str, args_json: &str) -> Result<String, String> {
    match function {
      "Sum" => {
        let args: serde_json::Value =
          serde_json::from_str(args_json).map_err(|e| e.to_string())?;
        let a = args[0].as_i64().ok_or("arg 0 not i64")?;
        let b = args[1].as_i64().ok_or("arg 1 not i64")?;
        Ok((a + b).to_string())
      }
      _ => Err(format!("unknown function: {}", function)),
    }
  }
}

register!(SumPlugin);
