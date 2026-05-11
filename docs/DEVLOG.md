# 开发问题记录

## Bevy 0.18 API 兼容性

### Opaque3d → Transparent3d

**问题**: Bevy 0.18 中 `Opaque3d` 渲染阶段改用 `ViewBinnedRenderPhases`，API 与 `ViewSortedRenderPhases` 完全不同，无法直接使用自定义 DrawCommand。

**解决**: 切换到 `Transparent3d` + `ViewSortedRenderPhases`，与官方 `custom_shader_instancing` 示例一致。

### `experimental_bevy_ui_widgets` 特性名

**问题**: Cargo.toml 中特性名写为 `bevy_ui_widgets`，实际正确名称为 `experimental_bevy_ui_widgets`。

**解决**: 修正为 `bevy = { path = "../../bevy", features = ["default", "experimental_bevy_ui_widgets"] }`。

### `SliderValue` 不可变组件

**问题**: `SliderValue` 带有 `#[component(immutable)]` 标记，无法使用 `&mut SliderValue` 查询修改值。ESC 重置滑块时需要修改滑块值。

**解决**: 使用 `commands.entity(entity).insert(SliderValue(0.0))` 替换组件。

### Bevy 0.18 Spawn API 变更

**问题**: 
- `Children::spawn()` 需要 `Spawn()` 包装每个子实体
- `Children::from_iter()` 不存在
- `ChildBuilder` 类型不可导入

**解决**: 改用 `commands.spawn()` + `add_child()` / `add_children()` 构建 UI 层级。

### `DerefMut` 未实现

**问题**: `#[derive(Deref)]` 只实现 `Deref`，使用 `**data = ...` 解引用赋值报错。

**解决**: 改用 `data.0 = visible.clone()` 直接访问字段。

### `Single` 系统参数过多

**问题**: `orbit_camera` 使用 `Single` 的 5 个参数导致系统配置失败。

**解决**: 将 Update 系统拆分为多个独立 `add_systems` 调用。

---

## 正方体不可见

### 根因 1: InstanceData.position.w = 0.0

**现象**: 应用运行但看不到任何正方体。

**原因**: WGSL 着色器使用 `vertex.position * vertex.i_pos_scale.w + vertex.i_pos_scale.xyz` 计算顶点位置，其中 `.w` 是缩放因子。`InstanceData.position.w` 设为 `0.0`，导致所有正方体缩放为零体积。

**解决**: 将 `position.w` 改为 `1.0`（`cube_grid.rs` 第 111、142 行）。

### 根因 2: 滑块初始化误触发 Changed

**现象**: 修复后只显示 1 个正方体（位于原点），日志显示实例数从 1,048,576 降到 1。

**原因**: `bevy_ui_widgets` 的 Slider 组件在初始化阶段内部修改 `SliderValue`，触发 `Changed<SliderValue>`。`on_slider_changed` 系统将其误认为用户操作，设置 `show_all = false` 并将所有滑块归零，导致 `compute_visible_instances` 只返回坐标 (0,0,0) 处的 1 个正方体。

**解决**: 在 `on_slider_changed` 中使用 `Local<bool>` 跳过首次触发。

---

## 视觉优化

### 纯色无光照导致面不可区分

**现象**: 正方体各面颜色完全相同，不同面之间、相邻正方体之间难以区分。

**解决**: 修改 WGSL 着色器添加：
1. **漫反射光照**: 根据法线方向计算亮度，不同朝向的面有明暗差异
2. **边缘暗化**: 计算 UV 到边缘距离，边缘处略微变暗（`smoothstep(0.0, 0.06, edge) * 0.2 + 0.8`）

---

## 交互冲突

### 滑块拖动时触发相机旋转

**现象**: 拖动滑块滑块时，视角会同时旋转。

**原因**: 鼠标左键同时被相机系统和滑块系统响应。

**解决**: 在 `orbit_camera` 中查询 `CoreSliderDragState`，检测到任何滑块正在拖动时跳过相机旋转。使用 `Query<&CoreSliderDragState, With<CubeGridSlider>>` 并检查 `dragging` 字段。

---

## 坐标约定调整

**问题**: 初始设计中 Y=16（上下）、Z=1024（深度）。用户采用 Y-up 坐标习惯，期望 Y=深度、Z=上下。

**解决**: 交换 `DIM_Y` 和 `DIM_Z` 常量定义，更新 `compute_grid_position` 中的世界坐标映射（参数 y→世界 Z，参数 z→世界 Y）。滑块范围自动适配。
