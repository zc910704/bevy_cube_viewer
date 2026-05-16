# 范围选择查看功能设计

## 概述

在现有截面模式基础上新增范围选择模式，允许用户指定 X/Y/Z 轴的区间范围来过滤显示正方体，而非仅限于单层截面。

## 设计决策

- **方案 A**：每轴 min/max 双滑块区间（采用）
- **交互**：模式切换（截面 / 范围），范围模式六条独立 Slider
- **替代关系**：范围模式完全替代截面模式 — 设 min=max 即等效截面

## 数据模型

`cross_section.rs` 中将 `CrossSectionState` 替换为 `RangeSelectionState`：

```rust
#[derive(Resource, Clone)]
pub struct RangeSelectionState {
    // 截面模式值（保留兼容）
    pub x_slider: u32, pub y_slider: u32, pub z_slider: u32,
    // 范围模式值
    pub x_min: u32, pub x_max: u32,
    pub y_min: u32, pub y_max: u32,
    pub z_min: u32, pub z_max: u32,
    pub mode: SelectionMode,
    pub dirty: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode { #[default] Section, Range }
```

- 默认值：`mode=Section`，全部 slider=0，min=0, max=DIM
- `max` 语义：0=全部显示不限制，1..=DIM 对应 0..DIM-1 层，与现有单滑块一致

## 过滤逻辑

`compute_visible_instances` 根据 `mode` 分支：

```rust
match state.mode {
    SelectionMode::Section => (
        axis_range(state.x_slider, DIM_X),
        axis_range(state.y_slider, DIM_Y),
        axis_range(state.z_slider, DIM_Z),
    ),
    SelectionMode::Range => (
        range_axis_range(state.x_min, state.x_max, DIM_X),
        range_axis_range(state.y_min, state.y_max, DIM_Y),
        range_axis_range(state.z_min, state.z_max, DIM_Z),
    ),
}
```

`range_axis_range(min, max, dim)`：`min=0` 从 0 开始，`max` 钳制到 dim，`min=max=N` 只包含 N-1 层。

## UI 结构

### 截面模式（保持现有）

- 3 条滑块（X/Y/Z），每轴一个 `SliderValue`
- 0 = 全部，1..=DIM = 单层

### 范围模式（新增）

- 6 条滑块：X-min、X-max、Y-min、Y-max、Z-min、Z-max
- 每条滑块标签显示当前数值（不可编辑）
- 每轴一行显示 "min ~ max / DIM"
- 底部显示选中方块数量

### 模式切换

- 模式切换按钮位于面板底部（截面模式 / 范围模式）
- 切换时控制两组滑块 Entity 的 `Visibility`
- 截面→范围时：用截面 slider 值初始化 min/max（单层变单点区间）
- 范围→截面时：保留范围值不变

### 布局

- 左下方面板：视图按钮 (3D/X/Y/Z剖面) + 模式切换 + 滑块
- 右下方面板：坐标显示（不变）
- 浮动标签：鼠标悬停（不变）

## 钳制规则

- min 滑块拖到 > max 时，max 自动设为 min
- max 滑块拖到 < min 时，min 自动设为 max
- 保证始终 min ≤ max

## ESC 重置

- 截面模式：x/y/z_slider = 0
- 范围模式：x/y/z_min = 0, x/y/z_max = DIM
- 两者都设置 `dirty = true`

## Picking 适配

`is_visible` 改为接受 `RangeSelectionState`，根据 mode 判断可见性。

## 实现范围

| 文件 | 改动 |
|------|------|
| `cube_grid.rs` | `CrossSectionState` → `RangeSelectionState`，新增 `range_axis_range` |
| `ui.rs` | 新增 6 条范围滑块 + 标签 + 模式切换按钮 + 可见性切换逻辑 |
| `main.rs` | 替换资源注册，新增 `on_mode_button_changed` 系统 |
| `picking.rs` | `is_visible` 适配新类型 |
| `cube_material.rs` | 类型名适配 |
| `camera.rs` | 类型名适配 |
