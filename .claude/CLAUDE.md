## Base Information
- Project: Bud - Next Generation Plug-in Foundation
- Language: Rust
- All documentation and comments should be written in **English**

## Crates Architecture

### shared-types (Foundation Layer)
Define core types and interfaces shared across all crates, including `Provider` trait and common data structures.

### wasm-provider (Provider Implementation Layer)
WASM Provider implementation based on wasmtime Component Model. Loads and executes WASM component plugins via the WIT-defined interface in `wit/bud.wit`.

### config (Configuration Management Layer)
Configuration file loading, parsing, and validation for both host application and plugin configurations.

### core (Core Runtime Layer)
Core runtime that integrates Provider and configuration management, providing the main BudCore API.

### test-harness (Testing Utilities)
Integration tests and example programs demonstrating how to use BudCore with WasmProvider.

### utils (Utility Functions)
Shared utilities crate that provides common helper tools and abstraction functions used across the project.

### xtask (Build Automation Layer)
Internal task runner crate for developer workflows that must keep WIT and WASM artifacts in sync.

- `cargo sync-wit`: sync `wit/bud.wit` to `sdk/rust/wit/bud.wit` (required for published SDK macro expansion)
- `cargo sdk-rust`: sync WIT and build `bud-plugin-sdk`
- `cargo sum-plugin`: sync WIT, build `example/sum-plugin`, and update `example/sum-plugin/main.wasm`
- `cargo plugin-dev`: one-command workflow that runs SDK build + sum-plugin WASM sync

## Plugin Interface Contract

`wit/bud.wit` is the **single source of truth** for the host↔plugin interface. All SDK bindings for every language are generated from this file by CI and published automatically.

```
wit/bud.wit
    ├── wasmtime::component::bindgen!()  →  host-side bindings (wasm-provider, compile-time)
    ├── wit-bindgen rust   →  bud-plugin-sdk (sdk/rust)         
    ├── wit-bindgen go     →  bud-plugin-sdk-go (Not realized)         
```

## Runtime Data Flow

```
Host App → BudCore::invoke(plugin, fn, args)
    → WasmProvider: args serialized to JSON string
    → WASM Component: on_invoke(fn, args_json) → result_json
    → WasmProvider: result deserialized to ProviderValue
    → Host App receives ProviderValue
```

JSON is used for invoke args/result because WIT does not support recursive variant types (`ProviderValue` can nest itself).

## Layered Architecture

```
┌─────────────────────────────────────┐
│        Host Application              │
│   Integrate plugin system via        │
│          BudCore                     │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Core Runtime Layer              │
│  BudCore + PluginManager + Config    │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│     Provider Abstraction Layer       │
│    WasmProvider (current) / future   │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Shared Types Layer              │
│     Provider trait + data types      │
└─────────────────────────────────────┘
```

## Development Guidelines

### Documentation Maintenance
- **CLAUDE.md Sync Rule**: When adding new crates or modifying existing crates (including changes to dependencies, key types, or responsibilities), this CLAUDE.md file MUST be updated accordingly to reflect the current architecture
- **Keep Architecture Accurate**: Ensure the layered architecture diagram and crate descriptions stay synchronized with actual code structure