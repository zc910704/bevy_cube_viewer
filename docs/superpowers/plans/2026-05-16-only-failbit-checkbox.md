# Only FailBit Checkbox Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 Range Mode 按钮右边添加 Only FailBit checkbox，勾选后只显示值为 1 的红色正方体。

**Architecture:** 在 `RangeSelectionState` 中新增 `only_failbit` bool 字段，遵循现有 dirty-flag 模式。`compute_visible_instances` 在迭代中跳过非红色方块。`build_button_bar` 中添加 Checkbox widget。新增系统监听 `ValueChange<bool>` 事件更新状态。ESC 重置时间步 checkbox UI。

**Tech Stack:** Rust, Bevy 0.18 ECS, bevy_ui_widgets::Checkbox

---

### Task 1: 添加 `only_failbit` 字段到 `RangeSelectionState`

**Files:**
- Modify: `src/cube_grid.rs`

- [ ] **Step 1: 在 `RangeSelectionState` 结构体中添加 `only_failbit` 字段**

```rust
// 在 RangeSelectionState struct 的 mode 字段之后添加:
pub only_failbit: bool,
```

- [ ] **Step 2: 在 `Default` 实现中初始化 `only_failbit: false`**

```rust
// 在 Default impl 的 mode: SelectionMode::default(), 之后添加:
only_failbit: false,
```

- [ ] **Step 3: 在 `compute_visible_instances` 循环中添加过滤逻辑**

在 `compute_visible_instances` 函数中，`let color = match grid.get(x, y, z) {` 之前添加：

```rust
if state.only_failbit && grid.get(x, y, z) != 1 {
    continue;
}
```

- [ ] **Step 4: 类型检查**

Run: `cargo check`
Expected: 编译通过

- [ ] **Step 5: Commit**

```bash
git add src/cube_grid.rs
git commit -m "feat: add only_failbit field to RangeSelectionState and filter in compute_visible_instances"
```

---

### Task 2: 添加 Checkbox UI 和事件处理

**Files:**
- Modify: `src/ui.rs`

- [ ] **Step 1: 添加 imports**

在 `src/ui.rs` 顶部的 `use bevy::ui_widgets::` 导入块中添加：

```rust
Checkbox, checkbox_self_update,
```
并在 `use bevy::ui::` 导入（或新增一行）添加：

```rust
Checkable,
```
在文件顶部添加新导入：

```rust
use bevy::ecs::observer::On;
use bevy::ui_widgets::ValueChange;
```

注意：`ValueChange` 需要合并到已有的 `ui_widgets` import 中。

- [ ] **Step 2: 添加 `FailBitCheckbox` 标记组件**

在 `ModeButton` 组件定义附近添加：

```rust
#[derive(Component)]
pub struct FailBitCheckbox;
```

- [ ] **Step 3: 在 `build_button_bar` 中添加 checkbox UI**

在 `build_button_bar` 函数中，在 `commands.entity(row).add_child(mode_btn);` 之后，添加 checkbox 构造代码：

```rust
let checkbox = commands
    .spawn((
        Checkbox,
        Checkable,
        FailBitCheckbox,
        Button, // 复用 Button 获得点击交互
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
        Text::new("Only FailBit"),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.85, 0.85)),
    ))
    .id();
commands.entity(row).add_child(checkbox);
```

- [ ] **Step 4: 添加 `on_failbit_changed` 观察者系统**

在 `ui.rs` 末尾添加新系统。使用 `On<ValueChange<bool>>` 接收 EntityEvent：

```rust
/// Updates only_failbit in state when checkbox is toggled.
pub fn on_failbit_changed(
    value_change: On<ValueChange<bool>>,
    checkbox: Query<(), With<FailBitCheckbox>>,
    mut state: ResMut<RangeSelectionState>,
) {
    if checkbox.contains(value_change.source) {
        state.only_failbit = value_change.value;
        state.dirty = true;
    }
}
```

- [ ] **Step 5: 适配 `update_cube_count`**

修改 `update_cube_count` 函数签名，添加 `grid: Res<CubeGrid>` 参数：

```rust
pub fn update_cube_count(
    state: Res<RangeSelectionState>,
    grid: Res<CubeGrid>,
    mut texts: Query<&mut Text, With<CubeCountText>>,
)
```

修改 Section 分支的计数逻辑，当 `state.only_failbit` 时遍历网格：

```rust
SelectionMode::Section => {
    let x_range = if state.x_slider == 0 { 0..DIM_X } else { (state.x_slider - 1) as usize..state.x_slider as usize };
    let y_range = if state.y_slider == 0 { 0..DIM_Y } else { (state.y_slider - 1) as usize..state.y_slider as usize };
    let z_range = if state.z_slider == 0 { 0..DIM_Z } else { (state.z_slider - 1) as usize..state.z_slider as usize };
    if state.only_failbit {
        let mut count = 0usize;
        for z in z_range {
            for y in y_range.clone() {
                for x in x_range.clone() {
                    if grid.get(x, y, z) == 1 {
                        count += 1;
                    }
                }
            }
        }
        count
    } else {
        x_range.len() * y_range.len() * z_range.len()
    }
}
```

Range 分支同样适配：

```rust
SelectionMode::Range => {
    let x_lo = state.x_min.saturating_sub(1) as usize;
    let x_hi = state.x_max.min(DIM_X as u32).saturating_sub(1) as usize;
    let y_lo = state.y_min.saturating_sub(1) as usize;
    let y_hi = state.y_max.min(DIM_Y as u32).saturating_sub(1) as usize;
    let z_lo = state.z_min.saturating_sub(1) as usize;
    let z_hi = state.z_max.min(DIM_Z as u32).saturating_sub(1) as usize;
    if state.only_failbit {
        let mut count = 0usize;
        for z in z_lo..=z_hi {
            for y in y_lo..=y_hi {
                for x in x_lo..=x_hi {
                    if grid.get(x, y, z) == 1 {
                        count += 1;
                    }
                }
            }
        }
        count
    } else {
        (x_hi - x_lo + 1) * (y_hi - y_lo + 1) * (z_hi - z_lo + 1)
    }
}
```

- [ ] **Step 6: 类型检查**

Run: `cargo check`
Expected: 编译通过

- [ ] **Step 7: Commit**

```bash
git add src/ui.rs
git commit -m "feat: add Only FailBit checkbox UI and event handling"
```

---

### Task 3: 主入口注册系统和 ESC 处理

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 更新 imports**

在 `use ui::` 导入块中添加新符号：

```rust
use ui::{
    // ... existing imports ...
    FailBitCheckbox, on_failbit_changed,
};
```

在 `use bevy::ui_widgets::` 导入中添加 `SetChecked`：

```rust
use bevy::ui_widgets::{UiWidgetsPlugins, SliderValue, SetChecked};
```

- [ ] **Step 2: 注册 `on_failbit_changed` 系统**

在 `main()` 中注册观察者（与 `add_systems` 同级）：

```rust
.add_observer(on_failbit_changed)
```

- [ ] **Step 3: ESC 处理中重置 `only_failbit`**

修改 `handle_esc` 函数签名，添加 checkbox 查询参数，并在各分支中重置 `state.only_failbit = false`。

完整修改后的 `handle_esc` 签名需要添加：

```rust
fn handle_esc(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<RangeSelectionState>,
    section_sliders: Query<Entity, (With<CubeGridSlider>, With<crate::ui::SliderAxis>)>,
    checkbox: Query<(Entity, Has<Checked>), With<FailBitCheckbox>>,
    mut commands: Commands,
)
```

在函数体内，修改 `SelectionMode::Section` 和 `SelectionMode::Range` 分支，在各自重置变量后都添加：

```rust
state.only_failbit = false;
```

然后在 `state.dirty = true;` 之后，同步 checkbox：

```rust
if let Ok((entity, is_checked)) = checkbox.get_single() {
    if is_checked {
        commands.trigger(SetChecked {
            entity,
            checked: false,
        });
    }
}
```

- [ ] **Step 4: 构建检查**

Run: `cargo check`
Expected: 编译通过

- [ ] **Step 5: Clippy 检查**

Run: `cargo clippy`
Expected: 无 warning

- [ ] **Step 6: 运行验证**

Run: `cargo run`
Expected: 应用启动，按钮栏出现 "Only FailBit" checkbox，勾选后只显示红色方块

- [ ] **Step 7: Commit**

```bash
git add src/main.rs
git commit -m "feat: register on_failbit_changed system and ESC reset for only_failbit"
```
