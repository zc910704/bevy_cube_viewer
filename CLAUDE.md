# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 构建与运行

```bash
cargo run
```

本项目依赖本地 Bevy 路径 (`../../bevy`)，确保 Bevy 仓库存在于正确的相对路径。

## 项目架构

这是一个 Bevy ECS 3D 应用程序，用于渲染和查看 3D 立方体网格。

### 核心模块

- **main.rs**: 应用入口，包含相机设置和轨道相机控制（鼠标左键旋转，滚轮缩放）
- **cube_renderer.rs**: `CubeRendererPlugin` 自定义插件，在启动时生成 3D 立方体网格阵列
- **shared_state.rs**: `SharedState` 资源，使用 `Arc<Mutex>` 在系统间共享状态

### 关键设计

- `CubeRendererPlugin` 使用 `Startup` 系统在应用启动时生成 75 个立方体（5×3×5 阵列）
- `SharedState` 提供跨系统共享的选中状态和请求坐标
- 轨道相机系统使用 `AccumulatedMouseMotion` 和 `MouseWheel` 处理输入

### 依赖

- `bevy`: 本地路径 `../../bevy`
- `bevy_diagnostic::LogDiagnosticsPlugin`: 诊断信息输出
