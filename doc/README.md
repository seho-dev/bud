# Bud - 插件框架设计文档

## 项目定位

Bud 是下一代跨平台应用扩展基础设施，支持宿主应用通过各种 Provider（WASM、Bun、其他运行时）来动态加载和管理插件。

## 核心特性

- **Provider 抽象**: 支持多种运行时环境（WASM、Bun、未来扩展）
- **权限系统**: 显式的权限声明和用户授权机制
- **清单驱动**: 通过 bud.json 描述插件元数据和权限
- **开发友好**: 提供 CLI 工具简化插件开发工作流

## 文档导航

### 架构与设计

- [核心概念](./core-concepts.md) - 理解 Bud 的基本术语和概念
- [Provider 架构](./provider-architecture.md) - Provider 层设计和扩展策略
- [插件系统](./plugin-system.md) - 插件清单、生命周期和权限管理

### 开发指南

- [开发工作流](./development-guide.md) - 插件开发和宿主集成完整指南

## 快速链接

- 项目 GitHub: [TODO]
- 示例插件: [TODO]
- CLI 工具: [TODO]
