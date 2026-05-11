# CLAUDE.md

## 构建与运行

```bash
cargo run
```

本项目依赖本地 Bevy 路径 (`../../bevy`)，确保 Bevy 仓库存在于正确的相对路径。

## 项目概述

Bevy ECS 3D 应用，使用 GPU Instancing 渲染 64×1024×16 = 1,048,576 个正方体阵列。

## 核心模块

- **main.rs**: 应用入口，插件注册，ESC 重置，系统编排
- **cube_grid.rs**: 网格数据 (`CubeGrid`, `CrossSectionState`)，坐标计算，可见性筛选
- **cube_material.rs**: 自定义渲染管线 (`CubeGridMaterialPlugin`)，GPU 实例缓冲，`DrawMeshInstanced` RenderCommand
- **ui.rs**: `bevy_ui_widgets::Slider` 横截面滑块 UI
- **camera.rs**: 轨道相机（鼠标左键旋转，滚轮缩放）

## 坐标约定

- X = 左右 (64)
- Y = 深度 (1024)
- Z = 上下 (16)

## 关键设计

- 单 Entity + 实例缓冲 = 单次 DrawCall 渲染所有正方体
- `Transparent3d` 渲染阶段（Bevy 0.18 `Opaque3d` 使用 `ViewBinnedRenderPhases` 不兼容）
- WGSL 着色器含漫反射光照 + 边缘暗化
- `NoFrustumCulling` 防止单个 Entity AABB 导致全部实例被错误剔除
- 滑块 0 = 该轴全显，1..DIM = 仅显示对应层
