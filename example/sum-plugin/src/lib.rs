use bud_plugin_sdk::bud::sdk::host::{emit, log, LogLevel};
use bud_plugin_sdk::{register, Plugin};

struct SumPlugin;

impl Plugin for SumPlugin {
  fn on_load() -> Result<(), String> {
    log(LogLevel::Info, "Sum plugin loaded");
    emit("test", "test payload");
    Ok(())
  }
}

register!(SumPlugin);
