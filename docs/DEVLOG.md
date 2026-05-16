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

**解决**: 将 `position.w` 改为 `1.0`。

### 根因 2: 滑块初始化误触发 Changed

**现象**: 修复后只显示 1 个正方体（位于原点），日志显示实例数从 1,048,576 降到 1。

**原因**: `bevy_ui_widgets` 的 Slider 组件在初始化阶段内部修改 `SliderValue`，触发 `Changed<SliderValue>`。`on_slider_changed` 系统将其误认为用户操作，导致滑块全部归零。

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

**现象**: 拖动滑块时，视角会同时旋转。

**原因**: 鼠标左键同时被相机系统和滑块系统响应。

**解决**: 在 `orbit_camera` 中查询 `CoreSliderDragState`，检测到任何滑块正在拖动时跳过相机旋转。使用 `Query<&CoreSliderDragState, With<CubeGridSlider>>` 并检查 `dragging` 字段。

---

## 坐标约定

**问题**: 初始设计中 Y=16（上下）、Z=1024（深度）。用户采用 Y-up 坐标习惯，期望 Y=深度、Z=上下。

**解决**: 交换 `DIM_Y` 和 `DIM_Z` 常量定义，更新 `compute_grid_position` 中的世界坐标映射（参数 y→世界 Z，参数 z→世界 Y）。

---

## 视图切换

### 剖面模式鼠标拖动方向反转

**现象**: 进入 X/Y/Z 剖面模式后，鼠标左键拖动时画面移动方向与预期相反。

**原因**: 剖面平移逻辑使用 `+=`，而 3D 轨道模式旋转的 delta 方向与平移期望的"拖拽世界"手感不一致。

**解决**: 将剖面平移的 delta 符号从 `+=` 改为 `-=`，使画面跟随鼠标拖动方向移动。

### 模式切换后的相机距离

**问题**: 从 3D 模式切换到剖面模式时，相机距离需要根据剖面可见范围自动调整，否则可能太近或太远。

**解决**: 在 `orbit_camera` 中检测 `ViewMode` 变化，根据当前剖面的两个可见维度计算合适的 `section_distance`：`visible_extent = max(dim1, dim2) * CUBE_SPACING`，`distance = (visible_extent / 2) / tan(fov / 2)`。

---

## 悬停拾取

### DDA 遍历坐标映射

**问题**: DDA 遍历使用网格坐标，步进逻辑需要正确映射 world 坐标轴到网格坐标轴（world Z → grid Y, world Y → grid Z）。

**关键代码注意**: `step_y` 基于 `dir.z`（world Z → grid Y），`step_z` 基于 `dir.y`（world Y → grid Z）。边界距离计算同样需要正确映射。

### 悬停高亮方案演进

**方案 A (CPU buffer 重建)**: 每帧检测 PickingState 变化，若有悬停目标则调用 `compute_visible_instances` 重建整个实例缓冲（32MB），将对应实例颜色替换为高亮色。每帧约 32MB CPU→GPU 数据传输。

**方案 C (shader uniform) — 当前方案**: 仅更新 16 字节 HoverUniform（bind group 3），顶点着色器中根据实例位置反算网格坐标，与 `hover_grid` 比较，匹配时替换颜色。消除每帧 CPU buffer 重建开销，数据传输从 ~32MB 降至 16B。

#### Shader 中的坐标反算

顶点着色器中根据 `i_pos_scale.xyz` 反算网格坐标：
```
gx = (pos.x / cube_spacing + (DIM_X - 1) / 2).round()
gy = (pos.z / cube_spacing + (DIM_Y - 1) / 2).round()  // world Z → grid Y
gz = (pos.y / cube_spacing + (DIM_Z - 1) / 2).round()  // world Y → grid Z
```
需要与 `compute_grid_position` (cube_grid.rs) 的映射保持精确一致。

### Camera 组件的 NoIndirectDrawing

**问题**: Bevy 0.18 `ViewSortedRenderPhases` 要求相机 Entity 带有 `NoIndirectDrawing` 组件，否则渲染阶段调度失败。

**解决**: `setup_camera` 中为相机 Entity 添加 `bevy::render::view::NoIndirectDrawing`。

### 视图切换时 section_target 重置

**问题**: 进入剖面模式时，`section_target` 应从 Vec3::ZERO 开始，而非保留上次剖面模式的偏移位置。

**解决**: 每次切换到剖面模式时将 `camera_state.section_target = Vec3::ZERO`。

---

## 范围选择功能 — B0001 Query 冲突

### 现象

添加范围选择功能（6 个范围滑块 + 模式切换按钮）后，应用启动即崩溃：

```
error[B0001]: Query<..., ...> in system <...> accesses component(s) <...>
in a way that conflicts with a previous system parameter.
```

崩溃发生在 `Update` schedule 内。

### 排查思路

1. **定位 schedule**: 从 backtrace 确认 panic 在 `Update` schedule 的 `schedule()` 调用中，缩小范围为 Update 系统的某个。

2. **缩小嫌疑人**: 新增/修改的系统有 7 个。先检查有多个 `Query` 参数的系统（B0001 本质是同一系统内两个 Query 访问了相同组件类型）。

3. **启用 trace feature**: `cargo run --features bevy/trace` 尝试获取系统名，但 trace feature 不足够，系统名仍显示为 `<Enable the debug feature to see the name>`。

4. **逐个检查**: 手动审查每个系统的 Query 参数，找出访问重叠组件的。

### 根因

Bevy 的 ECS 调度器在系统初始化时验证所有 Query 的组件访问是否兼容。**同一系统内两个 Query 访问相同组件类型即触发 B0001，即使两个都是只读（`&T`）访问也触发。** 这是因为使用了 `Changed<T>` filter 时内部会访问 `ChangeTrackers<T>`，Bevy 将两个 Query 访问同一组件类型的 change tracker 视为冲突。

涉及 3 个系统的 3 类冲突：

| 系统 | 冲突组件 | 原因 |
|------|----------|------|
| `on_slider_changed` | `SliderValue` | 两个 Query 都 `&SliderValue` + `Changed<SliderValue>` |
| `update_value_labels` | `SliderValue`, `Text` | 两个 slider Query 都访问 `SliderValue`；两个 text Query 都 `&mut Text` |
| `on_mode_button_changed` | `Visibility` | 两个 Query 都 `&mut Visibility` |

### 解决方法

**方法 A: `Option<>` 合并** — 当两个 Query 访问同一组件但用不同标记组件区分时：

```rust
// ❌ 错误: 两个 Query 都访问 SliderValue
section_sliders: Query<(&SliderValue, &SliderAxis), Changed<SliderValue>>,
range_sliders: Query<(&SliderValue, &RangeSliderAxis), Changed<SliderValue>>,

// ✅ 正确: 合并为一个，用 Option<> 替代标记组件
sliders: Query<(
    &SliderValue,
    Option<&SliderAxis>,
    Option<&RangeSliderAxis>,
), Changed<SliderValue>>,
```

```rust
// ❌ 错误: 两个 Query 都 &mut Text
mut texts: Query<(&mut Text, &SliderAxis), With<SliderValueText>>,
mut range_texts: Query<(&mut Text, &RangeSliderAxis), With<SliderValueText>>,

// ✅ 正确: 合并为 Option<>
mut texts: Query<(
    &mut Text,
    Option<&SliderAxis>,
    Option<&RangeSliderAxis>,
), With<SliderValueText>>,
```

**方法 B: `Has<>` 合并** — 当两个 Query 用 `With<A>` / `With<B>` 区分时：

```rust
// ❌ 错误: 两个 Query 都 &mut Visibility
mut section_panel: Query<&mut Visibility, (With<SectionSliderPanel>, Without<RangeSliderPanel>)>,
mut range_panel: Query<&mut Visibility, (With<RangeSliderPanel>, Without<SectionSliderPanel>)>,

// ✅ 正确: 合并为一个，用 Has<> 在循环内分支
mut panel_visibility: Query<(&mut Visibility, Has<SectionSliderPanel>)>,
// 循环内:
for (mut vis, is_section_panel) in &mut panel_visibility {
    *vis = if is_section_panel { Visibility::Visible } else { Visibility::Hidden };
}
```

### 关键要点

- B0001 是**编译期验证**但**运行时 panic**（Bevy 在系统初始化时检查，在第一次 schedule 执行时 panic）
- 两个 Query 即使都只读访问同一组件，只要有一个用了 `Changed<T>` filter，就会冲突
- 合并后逻辑更简洁：以前两个循环分别处理 section 和 range，现在一个循环内按 `Option<>` 分支
- `Has<T>` 比 `Option<&T>` 更高效（不需要实际读取组件数据），当只需要判断组件存在性时使用 `Has<T>`
