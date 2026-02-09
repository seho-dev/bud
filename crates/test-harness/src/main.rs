use core::BudCore;
use wasm_provider::WasmProvider;

fn main() {
  println!("\n=== Test 1: Using WasmProvider ===");
  let wasm_provider: WasmProvider = WasmProvider::new();
  match BudCore::builder(wasm_provider).build() {
    Ok(core) => {
      println!(
        "BudCore with WASM Provider initialized successfully! Config: {:?}",
        core.config
      )
    }
    Err(e) => println!("BudCore with WASM Provider initialization failed: {}", e),
  }
}
