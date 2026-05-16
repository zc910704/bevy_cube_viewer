# 范围选择查看功能 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在现有截面模式基础上新增范围选择模式，用六条独立 Slider 控制 X/Y/Z 轴的 min/max 区间过滤。

**Architecture:** `CrossSectionState` 替换为 `RangeSelectionState`，新增 `SelectionMode` 枚举。`compute_visible_instances` 和 `is_visible` 根据 mode 分支走截面/范围过滤。UI 通过模式切换按钮控制两组滑块的 Visibility。

**Tech Stack:** Rust, Bevy 0.18 (local path), bevy_ui_widgets::Slider

---

### 文件结构

| 文件 | 职责 | 改动类型 |
|------|------|----------|
| `cube_grid.rs` | RangeSelectionState, SelectionMode, 过滤逻辑 | 重构 |
| `picking.rs` | is_visible 适配新类型 | 小改 |
| `cube_material.rs` | 导入路径、类型名适配 | 小改 |
| `ui.rs` | 范围滑块 UI、模式切换、钳制逻辑 | 重改 |
| `main.rs` | 资源注册替换、handle_esc 扩展 | 小改 |
| `camera.rs` | 无改动 | — |

---

### Task 1: cube_grid.rs — 数据模型重构

**Files:**
- Modify: `src/cube_grid.rs`

- [ ] **Step 1: 替换 CrossSectionState 为 RangeSelectionState，新增 SelectionMode 和 range_axis_range**

将 `CrossSectionState` (line 61-79) 替换为：

```rust
/// Filter mode for cube visibility.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    #[default]
    Section,
    Range,
}

/// Grid filter state, controlled by UI sliders and ESC key.
/// Section mode: slider 0 = show all, 1..=DIM = single layer.
/// Range mode: min..=max, 0 and DIM = no limit on that side.
#[derive(Resource, Clone)]
pub struct RangeSelectionState {
    pub x_slider: u32,
    pub y_slider: u32,
    pub z_slider: u32,
    pub x_min: u32,
    pub x_max: u32,
    pub y_min: u32,
    pub y_max: u32,
    pub z_min: u32,
    pub z_max: u32,
    pub mode: SelectionMode,
    pub dirty: bool,
}

impl Default for RangeSelectionState {
    fn default() -> Self {
        Self {
            x_slider: 0, y_slider: 0, z_slider: 0,
            x_min: 0, x_max: DIM_X as u32,
            y_min: 0, y_max: DIM_Y as u32,
            z_min: 0, z_max: DIM_Z as u32,
            mode: SelectionMode::default(),
            dirty: true,
        }
    }
}
```

修改 `compute_visible_instances` (line 99)，签名改为接受 `RangeSelectionState`，内部按 mode 分支：

```rust
pub fn compute_visible_instances(
    grid: &CubeGrid,
    state: &RangeSelectionState,
) -> Vec<InstanceData> {
    let (x_range, y_range, z_range) = match state.mode {
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
    };

    let count = x_range.len() * y_range.len() * z_range.len();
    let mut out = Vec::with_capacity(count);

    for &z in &z_range {
        for &y in &y_range {
            for &x in &x_range {
                let pos = compute_grid_position(x, y, z);
                let color = match grid.get(x, y, z) {
                    1 => RED_COLOR,
                    2 => GRAY_COLOR,
                    _ => WHITE_COLOR,
                };
                out.push(InstanceData {
                    position: [pos.x, pos.y, pos.z, 1.0],
                    color,
                });
            }
        }
    }
    out
}
```

在 `axis_range` 下方新增 `range_axis_range`：

```rust
fn range_axis_range(min: u32, max: u32, dim: usize) -> Vec<usize> {
    let lo = (min.saturating_sub(1)) as usize;
    let hi = (max.min(dim as u32).saturating_sub(1)) as usize;
    (lo..=hi).collect()
}
```

- [ ] **Step 2: 运行 cargo check 验证编译**

Run: `cargo check 2>&1`
Expected: 大量编译错误（其他文件仍引用 `CrossSectionState`）

- [ ] **Step 3: 提交**

```bash
git add src/cube_grid.rs
git commit -m "refactor: CrossSectionState 替换为 RangeSelectionState，新增 SelectionMode"
```

---

### Task 2: picking.rs — is_visible 适配

**Files:**
- Modify: `src/picking.rs`

- [ ] **Step 1: 更新导入和函数签名**

Line 3-4: `CrossSectionState` → `RangeSelectionState`

Line 55-58: `dda_traverse` 参数 `cross_section: &CrossSectionState` → `state: &RangeSelectionState`

Line 102: `is_visible(grid.0, grid.1, grid.2, cross_section)` → `is_visible(grid.0, grid.1, grid.2, state)`

Line 138-150: `is_visible` 改为接受 `RangeSelectionState`，按 mode 分支：

```rust
fn is_visible(x: usize, y: usize, z: usize, state: &RangeSelectionState) -> bool {
    match state.mode {
        SelectionMode::Section => {
            (state.x_slider == 0 || x == (state.x_slider - 1) as usize)
            && (state.y_slider == 0 || y == (state.y_slider - 1) as usize)
            && (state.z_slider == 0 || z == (state.z_slider - 1) as usize)
        }
        SelectionMode::Range => {
            let x_lo = state.x_min.saturating_sub(1) as usize;
            let x_hi = state.x_max.min(DIM_X as u32).saturating_sub(1) as usize;
            let y_lo = state.y_min.saturating_sub(1) as usize;
            let y_hi = state.y_max.min(DIM_Y as u32).saturating_sub(1) as usize;
            let z_lo = state.z_min.saturating_sub(1) as usize;
            let z_hi = state.z_max.min(DIM_Z as u32).saturating_sub(1) as usize;
            x >= x_lo && x <= x_hi && y >= y_lo && y <= y_hi && z >= z_lo && z <= z_hi
        }
    }
}
```

Line 157: `picking_system` 参数 `cross_section: Res<CrossSectionState>` → `state: Res<RangeSelectionState>`

Line 178: `dda_traverse(ray.origin, *ray.direction, &cross_section)` → `dda_traverse(ray.origin, *ray.direction, &state)`

- [ ] **Step 2: 运行 cargo check 验证**

Run: `cargo check 2>&1`
Expected: picking.rs 通过，其余文件仍有错误

- [ ] **Step 3: 提交**

```bash
git add src/picking.rs
git commit -m "refactor: picking.rs 适配 RangeSelectionState"
```

---

### Task 3: cube_material.rs — 类型引用适配

**Files:**
- Modify: `src/cube_material.rs`

- [ ] **Step 1: 更新导入和类型引用**

Line 32: `CrossSectionState` → `RangeSelectionState`

Line 403-406: `update_instance_data` 签名中 `cross_section: Res<CrossSectionState>` → `state: Res<RangeSelectionState>`

Line 408: `cross_section.is_changed()` → `state.is_changed()`

Line 411: `compute_visible_instances(&grid, &cross_section)` → `compute_visible_instances(&grid, &state)`

- [ ] **Step 2: 运行 cargo check 验证**

Run: `cargo check 2>&1`
Expected: cube_material.rs 通过，uin 和 main 仍有错误

- [ ] **Step 3: 提交**

```bash
git add src/cube_material.rs
git commit -m "refactor: cube_material.rs 适配 RangeSelectionState"
```

---

### Task 4: ui.rs — 范围滑块组件和构建

**Files:**
- Modify: `src/ui.rs`

- [ ] **Step 1: 新增组件定义和常量**

在 `SliderAxis` 下方新增：

```rust
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum RangeSliderAxis {
    XMin, XMax,
    YMin, YMax,
    ZMin, ZMax,
}

#[derive(Component)]
pub struct ModeButton;

#[derive(Component)]
pub struct SectionSliderPanel;

#[derive(Component)]
pub struct RangeSliderPanel;

#[derive(Component)]
pub struct CubeCountText;
```

- [ ] **Step 2: 更新 imports**

```rust
use crate::cube_grid::{compute_grid_position, RangeSelectionState, SelectionMode, DIM_X, DIM_Y, DIM_Z};
```

- [ ] **Step 3: 新增 build_range_slider 辅助函数**

```rust
fn build_range_slider(
    commands: &mut Commands,
    axis: RangeSliderAxis,
    max: f32,
    default: f32,
    label: &str,
) -> Entity {
    let axis_label = commands
        .spawn((
            Text::new(label.to_string()),
            TextFont { font_size: 12.0, ..default() },
            TextColor(LABEL_COLOR),
        ))
        .id();

    let value_text = commands
        .spawn((
            Text::new(format!("{:.0}", default)),
            TextFont { font_size: 12.0, ..default() },
            TextColor(LABEL_COLOR),
            axis,
            SliderValueText,
        ))
        .id();

    let label_row = commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .add_children(&[axis_label, value_text])
        .id();

    let track = commands
        .spawn((
            Node {
                height: Val::Px(6.0),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(SLIDER_TRACK_COLOR),
        ))
        .id();

    let thumb = commands
        .spawn((
            CubeGridSliderThumb,
            SliderThumb,
            Node {
                display: Display::Flex,
                width: Val::Px(12.0),
                height: Val::Px(16.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(SLIDER_THUMB_COLOR),
        ))
        .id();

    let thumb_wrapper = commands
        .spawn(Node {
            display: Display::Flex,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(12.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        })
        .add_child(thumb)
        .id();

    let slider = commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Stretch,
                height: Val::Px(16.0),
                width: Val::Percent(100.0),
                ..default()
            },
            CubeGridSlider,
            axis,
            Slider { track_click: TrackClick::Snap },
            SliderValue(default),
            SliderRange::new(0.0, max),
            Hovered::default(),
            observe(slider_self_update),
        ))
        .add_children(&[track, thumb_wrapper])
        .id();

    commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(1.0),
            width: Val::Px(560.0),
            ..default()
        })
        .add_children(&[label_row, slider])
        .id()
}
```

- [ ] **Step 4: 修改 setup_ui，构建范围滑块面板和模式按钮**

在 `setup_ui` 中：

(1) 给现有截面滑块面板添加 `SectionSliderPanel` 标记组件：

```rust
// line 78: 给面板添加 SectionSliderPanel
commands
    .spawn((
        Node { ... },
        BackgroundColor(BG_COLOR),
        SectionSliderPanel,
    ))
```

(2) 构建范围滑块面板（隐藏）：

```rust
// 范围滑块面板（默认隐藏）
let x_min = build_range_slider(&mut commands, RangeSliderAxis::XMin, DIM_X as f32, 0.0, "min");
let x_max = build_range_slider(&mut commands, RangeSliderAxis::XMax, DIM_X as f32, DIM_X as f32, "max");
let y_min = build_range_slider(&mut commands, RangeSliderAxis::YMin, DIM_Y as f32, 0.0, "min");
let y_max = build_range_slider(&mut commands, RangeSliderAxis::YMax, DIM_Y as f32, DIM_Y as f32, "max");
let z_min = build_range_slider(&mut commands, RangeSliderAxis::ZMin, DIM_Z as f32, 0.0, "min");
let z_max = build_range_slider(&mut commands, RangeSliderAxis::ZMax, DIM_Z as f32, DIM_Z as f32, "max");

// 范围模式面板
commands
    .spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            left: Val::Px(16.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(12.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(BG_COLOR),
        Visibility::Hidden,
        RangeSliderPanel,
    ))
    .with_children(|parent| {
        // X axis group label + 2 sliders
        parent.spawn(Text::new("X 轴")).insert(TextFont { font_size: 13.0, ..default() }).insert(TextColor(Color::srgb(0.53, 0.76, 0.91)));
        parent.add_child(x_min);
        parent.add_child(x_max);
        // Y axis
        parent.spawn(Text::new("Y 轴")).insert(TextFont { font_size: 13.0, ..default() }).insert(TextColor(Color::srgb(0.53, 0.76, 0.91)));
        parent.add_child(y_min);
        parent.add_child(y_max);
        // Z axis
        parent.spawn(Text::new("Z 轴")).insert(TextFont { font_size: 13.0, ..default() }).insert(TextColor(Color::srgb(0.53, 0.76, 0.91)));
        parent.add_child(z_min);
        parent.add_child(z_max);
        // Cube count text
        parent.spawn((
            Text::new("显示 -- 方块"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.4, 0.7, 0.4)),
            CubeCountText,
        ));
    });
```

(3) 在 `build_button_bar` 中新增模式切换按钮：

在四个视图按钮之后、flex spacer 之前：

```rust
let mode_btn = commands
    .spawn((
        Button,
        Node {
            padding: UiRect::all(Val::Px(6.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(BTN_INACTIVE_COLOR),
        ModeButton,
        Text::new("范围模式"),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgb(0.85, 0.85, 0.85)),
    ))
    .id();
commands.entity(row).add_child(mode_btn);
```

- [ ] **Step 5: 运行 cargo check 验证**

Run: `cargo check 2>&1`
Expected: ui.rs 编译通过（可能会有未使用变量警告）

- [ ] **Step 6: 提交**

```bash
git add src/ui.rs
git commit -m "feat: 新增范围滑块构建和模式切换按钮"
```

---

### Task 5: ui.rs — 滑块同步、钳制、模式切换、方块计数逻辑

**Files:**
- Modify: `src/ui.rs`

- [ ] **Step 1: 重写 on_slider_changed，处理截面和范围两组滑块**

```rust
pub fn on_slider_changed(
    section_sliders: Query<(&SliderValue, &SliderAxis), Changed<SliderValue>>,
    range_sliders: Query<(&SliderValue, &RangeSliderAxis), Changed<SliderValue>>,
    mut state: ResMut<RangeSelectionState>,
    mut ready: Local<bool>,
) {
    if !*ready {
        *ready = true;
        return;
    }

    // Section sliders
    for (value, axis) in &section_sliders {
        let val = value.0 as u32;
        match axis {
            SliderAxis::X => state.x_slider = val,
            SliderAxis::Y => state.y_slider = val,
            SliderAxis::Z => state.z_slider = val,
        }
        state.dirty = true;
    }

    // Range sliders (with clamping)
    for (value, axis) in &range_sliders {
        let val = value.0 as u32;
        match axis {
            RangeSliderAxis::XMin => {
                state.x_min = val;
                if val > state.x_max { state.x_max = val; }
            }
            RangeSliderAxis::XMax => {
                state.x_max = val;
                if val < state.x_min { state.x_min = val; }
            }
            RangeSliderAxis::YMin => {
                state.y_min = val;
                if val > state.y_max { state.y_max = val; }
            }
            RangeSliderAxis::YMax => {
                state.y_max = val;
                if val < state.y_min { state.y_min = val; }
            }
            RangeSliderAxis::ZMin => {
                state.z_min = val;
                if val > state.z_max { state.z_max = val; }
            }
            RangeSliderAxis::ZMax => {
                state.z_max = val;
                if val < state.z_min { state.z_min = val; }
            }
        }
        state.dirty = true;
    }
}
```

- [ ] **Step 2: 新增 on_mode_button_changed 系统**

```rust
pub fn on_mode_button_changed(
    mut interaction_query: Query<(&Interaction, &ModeButton), Changed<Interaction>>,
    mut state: ResMut<RangeSelectionState>,
    mut section_panel: Query<&mut Visibility, (With<SectionSliderPanel>, Without<RangeSliderPanel>)>,
    mut range_panel: Query<&mut Visibility, (With<RangeSliderPanel>, Without<SectionSliderPanel>)>,
    mut mode_btns: Query<(&ModeButton, &mut Text), With<Button>>,
) {
    for (interaction, _) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // Toggle mode
        let new_mode = match state.mode {
            SelectionMode::Section => SelectionMode::Range,
            SelectionMode::Range => SelectionMode::Section,
        };
        state.mode = new_mode;
        state.dirty = true;

        // Toggle panel visibility
        let is_section = new_mode == SelectionMode::Section;
        for mut vis in &mut section_panel {
            *vis = if is_section { Visibility::Visible } else { Visibility::Hidden };
        }
        for mut vis in &mut range_panel {
            *vis = if is_section { Visibility::Hidden } else { Visibility::Visible };
        }

        // Update button text
        for (_, mut text) in &mut mode_btns {
            **text = if is_section {
                "范围模式".into()
            } else {
                "截面模式".into()
            };
        }
    }
}
```

- [ ] **Step 3: 新增 update_cube_count 系统**

```rust
pub fn update_cube_count(
    state: Res<RangeSelectionState>,
    mut texts: Query<&mut Text, With<CubeCountText>>,
) {
    if !state.is_changed() {
        return;
    }
    for mut text in &mut texts {
        let count = match state.mode {
            SelectionMode::Section => {
                let xc = if state.x_slider == 0 { DIM_X } else { 1 };
                let yc = if state.y_slider == 0 { DIM_Y } else { 1 };
                let zc = if state.z_slider == 0 { DIM_Z } else { 1 };
                xc * yc * zc
            }
            SelectionMode::Range => {
                let x_lo = state.x_min.saturating_sub(1) as usize;
                let x_hi = state.x_max.min(DIM_X as u32).saturating_sub(1) as usize;
                let y_lo = state.y_min.saturating_sub(1) as usize;
                let y_hi = state.y_max.min(DIM_Y as u32).saturating_sub(1) as usize;
                let z_lo = state.z_min.saturating_sub(1) as usize;
                let z_hi = state.z_max.min(DIM_Z as u32).saturating_sub(1) as usize;
                (x_hi - x_lo + 1) * (y_hi - y_lo + 1) * (z_hi - z_lo + 1)
            }
        };
        **text = format!("显示 {} 方块", count);
    }
}
```

- [ ] **Step 4: 扩展 update_value_labels 处理 RangeSliderAxis**

```rust
pub fn update_value_labels(
    section_sliders: Query<(&SliderValue, &SliderAxis), (Changed<SliderValue>, With<CubeGridSlider>)>,
    range_sliders: Query<(&SliderValue, &RangeSliderAxis), (Changed<SliderValue>, With<CubeGridSlider>)>,
    mut texts: Query<(&mut Text, &SliderAxis), With<SliderValueText>>,
    mut range_texts: Query<(&mut Text, &RangeSliderAxis), With<SliderValueText>>,
) {
    for (value, axis) in &section_sliders {
        for (mut text, txt_axis) in texts.iter_mut() {
            if axis == txt_axis {
                **text = format!("{:.0}", value.0);
            }
        }
    }
    for (value, axis) in &range_sliders {
        for (mut text, txt_axis) in range_texts.iter_mut() {
            if axis == txt_axis {
                **text = format!("{:.0}", value.0);
            }
        }
    }
}
```

- [ ] **Step 5: 新增 sync_range_sliders 系统（钳制后同步滑块位置到实际值）**

当 `RangeSelectionState` 的 min/max 被钳制修改后，需要将值写回 SliderValue 使 UI 一致：

```rust
pub fn sync_range_sliders(
    state: Res<RangeSelectionState>,
    mut sliders: Query<(&mut SliderValue, &RangeSliderAxis), With<CubeGridSlider>>,
) {
    if !state.is_changed() {
        return;
    }
    for (mut value, axis) in &mut sliders {
        let new_val = match axis {
            RangeSliderAxis::XMin => state.x_min as f32,
            RangeSliderAxis::XMax => state.x_max as f32,
            RangeSliderAxis::YMin => state.y_min as f32,
            RangeSliderAxis::YMax => state.y_max as f32,
            RangeSliderAxis::ZMin => state.z_min as f32,
            RangeSliderAxis::ZMax => state.z_max as f32,
        };
        if (value.0 - new_val).abs() > 0.5 {
            value.0 = new_val;
        }
    }
}
```

- [ ] **Step 6: 运行 cargo check 验证**

Run: `cargo check 2>&1`
Expected: ui.rs 编译通过（main.rs 仍有少许错误）

- [ ] **Step 7: 提交**

```bash
git add src/ui.rs
git commit -m "feat: 范围滑块同步、钳制、模式切换、方块计数系统"
```

---

### Task 6: main.rs — 资源注册和 ESC 处理

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 更新导入和资源注册**

Line 16: `CrossSectionState` → `RangeSelectionState`

Line 18: 新增导入：
```rust
use ui::{
    CubeGridSlider, setup_ui, update_slider_visuals, update_value_labels, on_slider_changed,
    on_view_button_changed, on_mode_button_changed, update_button_visuals,
    update_hover_coords_panel, update_hover_tooltip,
    update_cube_count, sync_range_sliders, SectionSliderPanel,
    RangeSliderPanel, ModeButton,
};
```

Line 35: `.init_resource::<CrossSectionState>()` → `.init_resource::<RangeSelectionState>()`

- [ ] **Step 2: 新增系统到 Update 调度**

在 `on_view_button_changed` 下方新增：
```rust
.add_systems(Update, on_mode_button_changed)
.add_systems(Update, update_cube_count)
.add_systems(Update, sync_range_sliders)
```

- [ ] **Step 3: 扩展 handle_esc**

```rust
fn handle_esc(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<RangeSelectionState>,
    slider_query: Query<Entity, With<CubeGridSlider>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.mode {
            SelectionMode::Section => {
                state.x_slider = 0;
                state.y_slider = 0;
                state.z_slider = 0;
            }
            SelectionMode::Range => {
                state.x_min = 0;
                state.x_max = DIM_X as u32;
                state.y_min = 0;
                state.y_max = DIM_Y as u32;
                state.z_min = 0;
                state.z_max = DIM_Z as u32;
            }
        }
        state.dirty = true;

        for entity in &slider_query {
            commands.entity(entity).insert(SliderValue(0.0));
        }
    }
}
```

- [ ] **Step 4: 运行 cargo check 验证全项目编译**

Run: `cargo check 2>&1`
Expected: 编译成功，无错误（可能有 unused import 警告）

- [ ] **Step 5: 清理 unused import 警告**

Run: `cargo clippy 2>&1`
如果有警告，清理之。

- [ ] **Step 6: 提交**

```bash
git add src/main.rs
git commit -m "feat: 集成范围选择功能 — 资源注册、系统调度、ESC 处理"
```

---

### Task 7: 运行验证和最终提交

- [ ] **Step 1: 运行 cargo clippy 最终检查**

Run: `cargo clippy 2>&1`
Expected: 零警告、零错误

- [ ] **Step 2: 运行 cargo run 手动验证**

Run: `cargo run 2>&1`

验证清单：
- [ ] 应用正常启动，可以看到正方体阵列
- [ ] 底部按钮栏有 "范围模式" 按钮（在四个视图按钮旁边）
- [ ] 点击 "范围模式" → 截面滑块面板隐藏，范围滑块面板显示
- [ ] 拖动 X-min 滑块，所选范围变小，方块数量更新
- [ ] 拖动 X-min 超过 X-max → X-max 自动跟随（钳制）
- [ ] 底部显示方块数量实时更新
- [ ] 点击 "截面模式" → 回到截面模式 UI
- [ ] ESC 键在两种模式下均正确重置
- [ ] 鼠标悬停拾取仍正常工作
- [ ] 剖面视图按钮（3D/X/Y/Z）仍正常工作

- [ ] **Step 3: 最终提交**

```bash
git add -u
git commit -m "chore: cargo clippy 修复"
```
