wit_bindgen::generate!({
    world: "bud-plugin",
    path: "wit/bud.wit",
    pub_export_macro: true,
});

#[doc(hidden)]
pub use exports as __exports;

pub trait Plugin {
  fn on_invoke(function: &str, _args_json: &str) -> Result<String, String> {
    Err(format!("unknown function: {}", function))
  }
}

#[macro_export]
macro_rules! register {
    ($t:ty) => {
        struct __BudGuestImpl;
        impl bud_plugin_sdk::__exports::bud::sdk::plugin::Guest for __BudGuestImpl {
            fn on_load() -> Result<(), String> {
                Ok(())
            }
            fn on_invoke(function: String, args_json: String) -> Result<String, String> {
                <$t as bud_plugin_sdk::Plugin>::on_invoke(&function, &args_json)
            }
        }
        bud_plugin_sdk::export!(__BudGuestImpl with_types_in bud_plugin_sdk);
    };
}
