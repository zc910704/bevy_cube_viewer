# Orbit Only Visible — 设计说明

## 目标

在 3D 模式下增加 "Orbit Only Visible" checkbox，勾选后轨道相机绕可见范围几何中心旋转（而非原点），关闭后恢复绕原点旋转。

## 改动

### camera.rs

- `CameraState` 新增 `orbit_only_visible: bool` 字段（默认 `false`）
- 新增 `fn visible_center(state: &RangeSelectionState) -> Vec3`：根据当前 mode 取各轴 min/max 的中点 grid 坐标，调用 `compute_grid_position` 返回世界坐标
- `orbit_camera` 新增 `cross_section: Res<RangeSelectionState>` 参数；3D 模式下 orbit target 根据 `orbit_only_visible` 选择 `visible_center()` 或 `Vec3::ZERO`

### ui.rs

- 新增 `OrbitVisibleCheckbox` marker component
- `build_button_bar` 中在 3D 按钮旁新增 checkbox（复用 Checkbox + Checkable + observe(checkbox_self_update) 模式），label 为 "Orbit Only Visible"
- 新增 `on_orbit_visible_changed` observer（`On<ValueChange<bool>>`），写入 `CameraState.orbit_only_visible`
- 新增 `update_orbit_checkbox_visuals` 系统（勾选时背景变绿，复用 BTN_ACTIVE_COLOR/BTN_INACTIVE_COLOR）

### main.rs

- `handle_esc` 中重置 `orbit_only_visible = false` 并 uncheck OrbitVisibleCheckbox
- 注册新 observer 和 system

### 不变

- 剖面模式不受影响
- `compute_visible_instances` / `RangeSelectionState` 不变
