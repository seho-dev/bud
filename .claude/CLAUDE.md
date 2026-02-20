## Base Information
- Project: Bud - Next Generation Plug-in Foundation
- Language: Rust
- All documentation and comments should be written in **English**

## Crates Architecture

### shared-types (Foundation Layer)
Define core types and interfaces shared across all crates, including `Provider` trait and common data structures.

### wasm-provider (Provider Implementation Layer)
WASM Provider implementation based on wasmtime for loading and executing WASM plugins.

### config (Configuration Management Layer)
Configuration file loading, parsing, and validation for both host application and plugin configurations.

### core (Core Runtime Layer)
Core runtime that integrates Provider and configuration management, providing the main BudCore API.

### test-harness (Testing Utilities)
Integration tests and example programs demonstrating how to use BudCore with WasmProvider.

### utils (Utility Functions)
Common utility functions shared across the project, including file system operations like recursive directory copying.

## Architecture Design

### Layered Architecture
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

### Core Features
- **Provider Neutral**: Support multiple runtimes through trait abstraction (currently WASM)
- **Configuration Driven**: Separate management of host config (config.json) and plugin config (plugin.json)
- **Generic Architecture**: BudCore accepts any Provider implementation via generic parameters
- **Modular Design**: Clear responsibilities and dependencies for each crate

## Development Guidelines

### Documentation Maintenance
- **CLAUDE.md Sync Rule**: When adding new crates or modifying existing crates (including changes to dependencies, key types, or responsibilities), this CLAUDE.md file MUST be updated accordingly to reflect the current architecture
- **Keep Architecture Accurate**: Ensure the layered architecture diagram and crate descriptions stay synchronized with actual code structure
