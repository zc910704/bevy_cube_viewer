# Orbit Only Visible Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 3D 模式下增加 "Orbit Only Visible" checkbox，勾选后轨道相机绕可见范围几何中心旋转而非绕原点。

**Architecture:** `CameraState` 新增 bool 标记；`camera.rs` 新增 `visible_center()` 根据 RangeSelectionState 计算几何中心；`ui.rs` 新增 checkbox 组件和 observer；`orbit_camera` 在 3D 模式下根据标记选择 orbit target。

**Tech Stack:** Bevy 0.18 ECS, bevy_ui_widgets Checkbox

---

### Task 1: 增加 orbit_only_visible 状态和可见中心计算

**Files:**
- Modify: `src/camera.rs`

- [ ] **Step 1: CameraState 新增 orbit_only_visible 字段**

在 `CameraState` 的 `Default` impl 中，在最后一个字段后添加：

```rust
pub orbit_only_visible: bool,
```

默认值 `false`。在 `section_target` 后加逗号改为非尾随字段：

```rust
impl Default for CameraState {
    fn default() -> Self {
        Self {
            orbit_distance: 80.0,
            section_distance: 200.0,
            section_target: Vec3::ZERO,
            orbit_only_visible: false,
        }
    }
}
```

- [ ] **Step 2: 新增 visible_center 辅助函数**

在 `impl ViewMode` 块之后、`impl Default for CameraState` 之前添加：

```rust
fn visible_center(state: &RangeSelectionState) -> Vec3 {
    use crate::cube_grid::SelectionMode;

    let (x_lo, x_hi) = match state.mode {
        SelectionMode::Section => axis_center_bounds(state.x_slider, DIM_X),
        SelectionMode::Range => range_center_bounds(state.x_min, state.x_max, DIM_X),
    };
    let (y_lo, y_hi) = match state.mode {
        SelectionMode::Section => axis_center_bounds(state.y_slider, DIM_Y),
        SelectionMode::Range => range_center_bounds(state.y_min, state.y_max, DIM_Y),
    };
    let (z_lo, z_hi) = match state.mode {
        SelectionMode::Section => axis_center_bounds(state.z_slider, DIM_Z),
        SelectionMode::Range => range_center_bounds(state.z_min, state.z_max, DIM_Z),
    };

    let x_mid = (x_lo + x_hi) as f32 / 2.0;
    let y_mid = (y_lo + y_hi) as f32 / 2.0;
    let z_mid = (z_lo + z_hi) as f32 / 2.0;

    Vec3::new(
        (x_mid - (DIM_X - 1) as f32 / 2.0) * CUBE_SPACING,
        (z_mid - (DIM_Z - 1) as f32 / 2.0) * CUBE_SPACING,
        (y_mid - (DIM_Y - 1) as f32 / 2.0) * CUBE_SPACING,
    )
}

fn axis_center_bounds(slider: u32, dim: usize) -> (usize, usize) {
    if slider == 0 {
        (0, dim - 1)
    } else {
        let v = (slider - 1) as usize;
        (v, v)
    }
}

fn range_center_bounds(min: u32, max: u32, dim: usize) -> (usize, usize) {
    let lo = min.saturating_sub(1) as usize;
    let hi = max.min(dim as u32).saturating_sub(1) as usize;
    (lo, hi)
}
```

`visible_center` 为公开函数，供 `orbit_camera` 调用。

### Task 2: 修改 orbit_camera 使用可见中心

**Files:**
- Modify: `src/camera.rs:53-116`

- [ ] **Step 1: orbit_camera 新增 cross_section 参数**

在 `orbit_camera` 函数签名中，在 `view_mode: Res<ViewMode>,` 之后添加：

```rust
cross_section: Res<RangeSelectionState>,
```

- [ ] **Step 2: 3D 模式下根据 orbit_only_visible 选择 target**

将第 114 行：

```rust
let target = Vec3::ZERO;
```

替换为：

```rust
let target = if camera_state.orbit_only_visible {
    visible_center(&cross_section)
} else {
    Vec3::ZERO
};
```

### Task 3: 新增 OrbitVisibleCheckbox UI 组件

**Files:**
- Modify: `src/ui.rs`

- [ ] **Step 1: 新增 OrbitVisibleCheckbox marker component**

在 `FailBitCheckbox` 之后添加：

```rust
#[derive(Component)]
pub struct OrbitVisibleCheckbox;
```

- [ ] **Step 2: build_button_bar 中添加 checkbox**

在 `build_button_bar` 函数中，3D 按钮的 add_child 之后、mode_btn 创建之前，添加 orbit checkbox：

```rust
let orbit_checkbox = commands
    .spawn((
        Checkbox,
        Checkable,
        OrbitVisibleCheckbox,
        Button,
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(6.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(BTN_INACTIVE_COLOR),
        observe(checkbox_self_update),
    ))
    .with_child((
        Text::new("Orbit Only Visible"),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.85, 0.85)),
    ))
    .id();
commands.entity(row).add_child(orbit_checkbox);
```

### Task 4: observer 和 checkbox 视觉系统

**Files:**
- Modify: `src/ui.rs`

- [ ] **Step 1: 新增 on_orbit_visible_changed observer**

在文件末尾添加：

```rust
pub fn on_orbit_visible_changed(
    value_change: On<ValueChange<bool>>,
    checkbox: Query<(), With<OrbitVisibleCheckbox>>,
    mut camera_state: ResMut<CameraState>,
) {
    if checkbox.contains(value_change.source) {
        camera_state.orbit_only_visible = value_change.value;
    }
}
```

注意需要 `use crate::camera::CameraState;`，检查现有 import 是否已包含。

- [ ] **Step 2: 新增 update_orbit_checkbox_visuals 系统**

```rust
pub fn update_orbit_checkbox_visuals(
    checkbox: Query<Has<Checked>, With<OrbitVisibleCheckbox>>,
    mut bg: Query<&mut BackgroundColor, With<OrbitVisibleCheckbox>>,
) {
    if let Ok(is_checked) = checkbox.single() {
        if let Ok(mut bg) = bg.single_mut() {
            bg.0 = if is_checked {
                BTN_ACTIVE_COLOR
            } else {
                BTN_INACTIVE_COLOR
            };
        }
    }
}
```

### Task 5: 注册新系统和 observer，ESC 重置

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 注册 observer 和 system**

在 `add_observer(on_failbit_changed)` 之后添加：

```rust
.add_observer(ui::on_orbit_visible_changed)
```

在 `add_systems(Update, update_checkbox_visuals)` 之后添加：

```rust
.add_systems(Update, ui::update_orbit_checkbox_visuals)
```

- [ ] **Step 2: handle_esc 增加 OrbitVisibleCheckbox 重置**

`handle_esc` 函数需要新增两个参数：

```rust
mut camera_state: ResMut<CameraState>,
orbit_checkbox: Query<(Entity, Has<Checked>), With<OrbitVisibleCheckbox>>,
```

在 `state.dirty = true;` 之前添加：

```rust
camera_state.orbit_only_visible = false;
```

在现有 failbit checkbox 重置逻辑之后，添加 orbit checkbox 的 uncheck：

```rust
if let Ok((entity, is_checked)) = orbit_checkbox.single() {
    if is_checked {
        commands.trigger(SetChecked {
            entity,
            checked: false,
        });
    }
}
```

### Task 6: 编译验证并提交

**Files:**
- None (验证步骤)

- [ ] **Step 1: 编译检查**

```bash
cargo check
```
Expected: 编译成功，无错误无 warning

- [ ] **Step 2: Clippy 检查**

```bash
cargo clippy
```
Expected: 无新 warning

- [ ] **Step 3: 运行验证**

```bash
cargo run
```
手工验证：
1. 启动后 checkbox 默认未选中，轨道绕原点旋转
2. 勾选 "Orbit Only Visible"，旋转摄像机观察旋转中心是否偏移到可见范围中心
3. 拖动 slider 改变可见范围，旋转中心随之更新
4. 取消勾选，旋转中心恢复原点
5. ESC 重置时 checkbox 取消选中

- [ ] **Step 4: 提交**

```bash
git add src/camera.rs src/ui.rs src/main.rs
git commit -m "feat: add Orbit Only Visible checkbox for 3D orbit center"
```
