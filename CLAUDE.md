# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 构建与检查

```bash
cargo run              # 构建并运行
cargo check            # 快速类型检查（不生成二进制）
cargo build            # 仅构建
cargo clippy           # Lint 检查
```

本项目依赖本地 Bevy 路径 (`../../bevy`)，确保 Bevy 仓库存在于正确的相对路径。使用 git worktree 时需创建 bevy 符号链接：
```bash
ln -s /mnt/code/rust/bevy .worktrees/<worktree-name>/bevy
```

## 项目概述

Bevy ECS 3D 应用，使用 GPU Instancing 渲染 64×1024×16 = 1,048,576 个正方体阵列。
基于 Bevy 0.18 (本地路径)，启用 `experimental_bevy_ui_widgets` feature。

## Cargo 依赖

| 依赖 | 用途 |
|------|------|
| `bevy` (path) | ECS 引擎，含 `experimental_bevy_ui_widgets` feature |
| `bevy_dev_tools` (path) | `FpsOverlayPlugin` 右上角 FPS 显示 |
| `bytemuck` | 零拷贝 GPU 实例数据（`Pod` + `Zeroable` derive） |

## 核心模块

- **main.rs**: 应用入口，插件注册，ESC 重置滑块，系统编排
- **cube_grid.rs**: 网格数据 (`CubeGrid`), 横截面状态 (`CrossSectionState`), 坐标计算, 可见性筛选
- **cube_material.rs**: 自定义渲染管线 (`CubeGridMaterialPlugin`)，GPU 实例缓冲，`DrawMeshInstanced` RenderCommand
- **ui.rs**: `bevy_ui_widgets::Slider` 横截面滑块 UI (X/Y/Z 三轴)
- **camera.rs**: 轨道相机（鼠标左键旋转，滚轮缩放；拖动滑块时屏蔽旋转）
- **assets/shaders/cube_grid.wgsl**: 自定义 WGSL 着色器（漫反射光照 + 边缘暗化）

## 坐标约定

网格坐标 (数组索引) 与世界坐标的映射 — **注意 Y/Z 互换**：

| 轴 | 网格维度 | 网格含义 | 世界坐标 | 说明 |
|----|----------|----------|----------|------|
| X  | DIM_X=64 | 左右 | world X | 一致 |
| Y  | DIM_Y=1024 | 深度 | **world Z** | Y→Z 映射 |
| Z  | DIM_Z=16 | 上下 | **world Y** | Z→Y 映射 |

即：`compute_grid_position` 将 `(grid_x, grid_y, grid_z)` 映射为 `world(x, z_grid, y_grid)`。

## 数据流 — GPU Instancing 架构

```
[main world]                             [render world]
    │                                         │
CrossSectionState (dirty flag)                │
    │                                         │
    ▼                                         │
compute_visible_instances() → Vec<InstanceData>
    │                                         │
    ▼                                         │
InstanceMaterialData (Component) ──Extract──→ InstanceMaterialData
                                                    │
                                           prepare_instance_buffers
                                                    │
                                                    ▼
                                             InstanceBuffer (GPU)
                                                    │
                                           DrawMeshInstanced
                                                    │
                                                    ▼
                                              单次 DrawCall
```

## 关键 RenderCommand

`DrawCubeGrid` 是 type alias:

```rust
type DrawCubeGrid = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetMeshBindGroup<2>,
    DrawMeshInstanced,   // 自定义：set vertex buffer 0 (mesh) + 1 (instance), draw_indexed
);
```

## InstanceData 内存布局

32 bytes, 16 字节对齐 (适配 WGSL vec4)：

| 偏移 | 字段 | 类型 |
|------|------|------|
| 0 | position (xyz, w=unused) | `[f32; 4]` |
| 16 | color (rgba) | `[f32; 4]` |

WGSL shader locations: `@location(3)` = i_pos_scale, `@location(4)` = i_color.

## 横截面滑块逻辑

- 滑块值 `0` = 该轴显示所有层
- 滑块值 `1..=DIM` = 仅显示该轴第 N 层（1-indexed 映射到 `N-1`）
- `on_slider_changed` 写入 `CrossSectionState` 并设置 `dirty = true`
- 跳过首次 `Changed<SliderValue>` 以避免初始化时的误触发
- `update_instance_data` 检测 `dirty` 后调用 `compute_visible_instances` 重建实例数据
- ESC 键将所有滑块重置为 0，设置 `dirty = true`

## Bevy 0.18 特定注意事项

- 渲染阶段使用 `Transparent3d` 而非 `Opaque3d`，因为 `ViewBinnedRenderPhases` 与实例化单 Entity 不兼容
- 使用 `NoFrustumCulling` 防止单个 Entity 的 AABB 导致大量实例被错误剔除
- 使用 `RenderApp` 架构：main world 组件通过 `ExtractComponentPlugin` 抽取到 render world
- Mesh 索引缓冲区通过 `MeshAllocator` / `mesh_allocator.mesh_vertex_slice()` 访问（非直接 buffer 引用）
