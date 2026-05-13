# View Switching 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 添加 4 个视角切换按钮（3D/X/Y/Z 剖面），剖面模式下摄像机正对剖面平面，支持平移和缩放手势。

**Architecture:** 新增 `ViewMode` Resource 管理视角状态，扩展 `CameraState` 存储剖面摄像参数，`orbit_camera` 系统按模式分支处理输入。UI 新增独立按钮面板。

**Tech Stack:** Rust, Bevy 0.18 ECS, `bevy_ui_widgets`

---

### Task 1: camera.rs — 新增 ViewMode 和扩展 CameraState

**Files:**
- Modify: `src/camera.rs`

- [ ] **Step 1: 添加 ViewMode 枚举和 CameraState 新字段**

在 `src/camera.rs` 中，`CameraState` 定义之前添加 `ViewMode`：

```rust
/// Camera view mode: 3D orbit or orthogonal section views.
#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    ThreeD,
    SectionX,
    SectionY,
    SectionZ,
}

impl Default for ViewMode {
    fn default() -> Self {
        Self::ThreeD
    }
}
```

将 `CameraState` 改为：

```rust
#[derive(Resource)]
pub struct CameraState {
    pub orbit_distance: f32,
    pub section_distance: f32,
    pub section_target: Vec3,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            orbit_distance: 80.0,
            section_distance: 200.0,
            section_target: Vec3::ZERO,
        }
    }
}
```

- [ ] **Step 2: 添加辅助函数：计算剖面模式下的固定朝向和 up 向量**

在 `CameraState` 下方添加：

```rust
impl ViewMode {
    /// Returns (forward_direction, up_vector) for the section view camera.
    /// Both are in world space. Forward points from camera toward target.
    pub fn section_orientation(&self) -> (Vec3, Vec3) {
        match self {
            ViewMode::ThreeD => (Vec3::NEG_Z, Vec3::Y), // unused
            ViewMode::SectionX => (Vec3::NEG_X, Vec3::Y),
            ViewMode::SectionY => (Vec3::NEG_Z, Vec3::Y),
            ViewMode::SectionZ => (Vec3::NEG_Y, Vec3::Z),
        }
    }
}
```

- [ ] **Step 3: 运行 cargo check 验证编译**

```bash
cargo check 2>&1
```

预期：通过（尚未使用新类型，不引入编译错误）。

- [ ] **Step 4: Commit**

```bash
git add src/camera.rs
git commit -m "feat: 添加 ViewMode 枚举和 CameraState 剖面字段"
```

---

### Task 2: camera.rs — 实现剖面模式摄像逻辑

**Files:**
- Modify: `src/camera.rs`

- [ ] **Step 1: 修改 orbit_camera 函数签名，增加 ViewMode 和 Projection 参数**

将 `orbit_camera` 的 system params 改为：

```rust
pub fn orbit_camera(
    mut camera: Single<(&mut Transform, &Projection), With<Camera>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    mut camera_state: ResMut<CameraState>,
    view_mode: Res<ViewMode>,
    slider_drag: Query<&CoreSliderDragState, With<CubeGridSlider>>,
) {
    let (mut transform, projection) = camera.into_inner();
    // ...
}
```

- [ ] **Step 2: 提取 FOV 和剖面模式常量**

在 `orbit_camera` 函数开头添加：

```rust
let fov = match projection {
    Projection::Perspective(p) => p.fov,
    _ => std::f32::consts::consts::FRAC_PI_3,
};
```

从 `cube_grid` 引入常量，计算每种剖面模式的适配距离。在 `src/camera.rs` 顶部添加：

```rust
use crate::cube_grid::{DIM_X, DIM_Y, DIM_Z, CUBE_SPACING};
```

- [ ] **Step 3: 实现模式过渡逻辑**

在 `orbit_camera` 中添加 `Local<Option<ViewMode>>` 和过渡处理：

```rust
fn orbit_camera(
    ...
    mut prev_mode: Local<Option<ViewMode>>,
    ...
) {
    ...
    let mode = *view_mode;
    let mode_changed = *prev_mode != Some(mode);
    if mode_changed {
        *prev_mode = Some(mode);
    }
    ...
```

当 `mode_changed` 且新模式为 Section 时：

```rust
if mode_changed {
    if mode != ViewMode::ThreeD {
        // 进入剖面模式：计算适配距离
        let visible_extent = match mode {
            ViewMode::SectionX => (DIM_Y as f32 * CUBE_SPACING).max(DIM_Z as f32 * CUBE_SPACING),
            ViewMode::SectionY => (DIM_X as f32 * CUBE_SPACING).max(DIM_Z as f32 * CUBE_SPACING),
            ViewMode::SectionZ => (DIM_X as f32 * CUBE_SPACING).max(DIM_Y as f32 * CUBE_SPACING),
            _ => 100.0,
        };
        camera_state.section_distance = (visible_extent / 2.0) / (fov / 2.0).tan();
        camera_state.section_target = Vec3::ZERO;
    } else {
        // 返回 3D：从当前 transform 反算 orbit_distance
        camera_state.orbit_distance = transform.translation.distance(Vec3::ZERO);
    }
}
```

- [ ] **Step 4: 实现剖面模式的每帧逻辑**

在 `orbit_camera` 中添加模式分支，替换原有逻辑：

```rust
let dragging_slider = slider_drag.iter().any(|d| d.dragging);

match mode {
    ViewMode::ThreeD => {
        // 现有轨道相机逻辑
        let delta = mouse_motion.delta;
        if mouse_buttons.pressed(MouseButton::Left) && !dragging_slider {
            let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
            let new_yaw = yaw - delta.x * 0.004;
            let new_pitch = (pitch - delta.y * 0.003).clamp(-1.5, 1.5);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, roll);
        }
        for event in mouse_wheel_reader.read() {
            camera_state.orbit_distance =
                (camera_state.orbit_distance - event.y * 5.0).clamp(10.0, 2000.0);
        }
        let target = Vec3::ZERO;
        transform.translation = target - transform.forward() * camera_state.orbit_distance;
    }
    ViewMode::SectionX | ViewMode::SectionY | ViewMode::SectionZ => {
        // 设置固定朝向
        let (forward, up) = mode.section_orientation();
        transform.look_at(camera_state.section_target + forward, up);

        // 鼠标左键拖动 = 平移
        if mouse_buttons.pressed(MouseButton::Left) && !dragging_slider {
            let right = transform.rotation * Vec3::X;
            let cam_up = transform.rotation * Vec3::Y;
            let speed = camera_state.section_distance * 0.002;
            camera_state.section_target +=
                right * mouse_motion.delta.x * speed
                - cam_up * mouse_motion.delta.y * speed;
        }

        // 滚轮 = 缩放
        for event in mouse_wheel_reader.read() {
            camera_state.section_distance =
                (camera_state.section_distance - event.y * 5.0).clamp(10.0, 2000.0);
        }

        // 计算摄像机位置
        transform.translation =
            camera_state.section_target - forward * camera_state.section_distance;
    }
}
```

- [ ] **Step 5: 运行 cargo check 验证编译**

```bash
cargo check 2>&1
```

预期：通过。

- [ ] **Step 6: Commit**

```bash
git add src/camera.rs
git commit -m "feat: 实现剖面模式摄像逻辑（平移+缩放）"
```

---

### Task 3: ui.rs — 添加视角按钮面板和交互系统

**Files:**
- Modify: `src/ui.rs`

- [ ] **Step 1: 添加 ViewButtonAxis 组件和按钮 color 常量**

在 `src/ui.rs` 顶部现有常量区域添加：

```rust
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ViewButtonAxis {
    ThreeD,
    X,
    Y,
    Z,
}

const BTN_INACTIVE_COLOR: Color = Color::srgb(0.1, 0.1, 0.12);
const BTN_ACTIVE_COLOR: Color = Color::srgb(0.4, 0.7, 0.4);
const BTN_HOVER_COLOR: Color = Color::srgb(0.6, 0.85, 0.6);
```

- [ ] **Step 2: 添加 build_button_bar 函数**

在 `build_slider` 函数之后添加：

```rust
fn build_button_bar(commands: &mut Commands) -> Entity {
    let row = commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(4.0),
            ..default()
        })
        .id();

    let buttons = [
        (ViewButtonAxis::ThreeD, "3D"),
        (ViewButtonAxis::X, "X 剖面"),
        (ViewButtonAxis::Y, "Y 剖面"),
        (ViewButtonAxis::Z, "Z 剖面"),
    ];

    for (axis, label) in buttons {
        let btn = commands
            .spawn((
                Button,
                Node {
                    padding: UiRect::all(Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(BTN_INACTIVE_COLOR),
                axis,
                Text::new(label),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
            ))
            .id();
        commands.entity(row).add_child(btn);
    }

    row
}
```

- [ ] **Step 3: 修改 setup_ui，在滑块面板上方添加按钮栏**

修改 `setup_ui` 函数。在 `let panel = commands.spawn(...)` 之后、添加 slider children 之前，添加按钮栏。将按钮栏和滑块面板放入一个外层垂直容器：

```rust
pub fn setup_ui(mut commands: Commands) {
    // 按钮面板
    let button_bar = commands
        .spawn((
            Node {
                padding: UiRect::all(Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BG_COLOR),
        ))
        .add_child(build_button_bar(&mut commands))
        .id();

    // 滑块面板（现有逻辑，保持不变）
    let slider_panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BG_COLOR),
        ))
        .id();

    let x_slider = build_slider(&mut commands, SliderAxis::X, DIM_X as f32, "X");
    let y_slider = build_slider(&mut commands, SliderAxis::Y, DIM_Y as f32, "Y");
    let z_slider = build_slider(&mut commands, SliderAxis::Z, DIM_Z as f32, "Z");

    commands.entity(slider_panel).add_children(&[x_slider, y_slider, z_slider]);

    // 按钮栏定位在滑块面板上方
    commands.entity(button_bar).insert(Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(16.0 + 150.0), // 滑块面板约 150px 高，按钮栏在其上方
        left: Val::Px(16.0),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(12.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default()
    });
}
```

> **注意：** 固定偏移值 150px 是估值。如果实际运行时位置不理想，后续调整。

- [ ] **Step 4: 添加按钮交互系统**

在文件末尾添加两个新系统：

```rust
/// Sets ViewMode when a view button is pressed.
pub fn on_view_button_changed(
    mut interaction_query: Query<
        (&Interaction, &ViewButtonAxis),
        Changed<Interaction>,
    >,
    mut view_mode: ResMut<crate::camera::ViewMode>,
) {
    for (interaction, axis) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        *view_mode = match axis {
            ViewButtonAxis::ThreeD => crate::camera::ViewMode::ThreeD,
            ViewButtonAxis::X => crate::camera::ViewMode::SectionX,
            ViewButtonAxis::Y => crate::camera::ViewMode::SectionY,
            ViewButtonAxis::Z => crate::camera::ViewMode::SectionZ,
        };
    }
}

/// Updates button background color to reflect active view mode.
pub fn update_button_visuals(
    view_mode: Res<crate::camera::ViewMode>,
    mut buttons: Query<(&ViewButtonAxis, &mut BackgroundColor, &Interaction)>,
) {
    for (axis, mut bg, interaction) in &mut buttons {
        let is_active = match (*view_mode, axis) {
            (crate::camera::ViewMode::ThreeD, ViewButtonAxis::ThreeD) => true,
            (crate::camera::ViewMode::SectionX, ViewButtonAxis::X) => true,
            (crate::camera::ViewMode::SectionY, ViewButtonAxis::Y) => true,
            (crate::camera::ViewMode::SectionZ, ViewButtonAxis::Z) => true,
            _ => false,
        };
        bg.0 = if is_active {
            BTN_ACTIVE_COLOR
        } else if *interaction == Interaction::Hovered {
            BTN_HOVER_COLOR
        } else {
            BTN_INACTIVE_COLOR
        };
    }
}
```

- [ ] **Step 5: 运行 cargo check 验证编译**

```bash
cargo check 2>&1
```

预期：通过。

- [ ] **Step 6: Commit**

```bash
git add src/ui.rs
git commit -m "feat: 添加视角切换按钮面板和交互系统"
```

---

### Task 4: main.rs — 注册新 Resource 和系统

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 导入新类型**

在 `src/main.rs` 顶部，修改 imports：

```rust
use camera::{CameraState, ViewMode, orbit_camera};
```

```rust
use ui::{
    CubeGridSlider, setup_ui, update_slider_visuals, update_value_labels, on_slider_changed,
    on_view_button_changed, update_button_visuals,
};
```

- [ ] **Step 2: 注册 ViewMode Resource 和新系统**

在 `App::new()` 链中添加：

```rust
.init_resource::<ViewMode>()
```

并在 `Update` systems 中添加：

```rust
.add_systems(Update, on_view_button_changed)
.add_systems(Update, update_button_visuals)
```

这些系统应与现有 UI 系统放在一起：

```rust
.add_systems(Update, on_slider_changed)
.add_systems(Update, update_instance_data)
.add_systems(Update, update_slider_visuals)
.add_systems(Update, update_value_labels)
.add_systems(Update, on_view_button_changed)       // 新增
.add_systems(Update, update_button_visuals)        // 新增
.add_systems(Update, handle_esc)
```

- [ ] **Step 3: 运行 cargo check 验证编译**

```bash
cargo check 2>&1
```

预期：通过。

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: 注册 ViewMode Resource 和按钮交互系统"
```

---

### Task 5: 运行完整验证

- [ ] **Step 1: 构建**

```bash
cargo build 2>&1
```

预期：通过。

- [ ] **Step 2: 运行 clippy**

```bash
cargo clippy 2>&1
```

预期：无新 warning。

- [ ] **Step 3: 运行应用手动验证**

```bash
cargo run
```

验证清单：
- [ ] 底部左侧显示两个面板：按钮栏在上，滑块栏在下
- [ ] 默认 "3D" 按钮高亮（绿色）
- [ ] 点击 "X 剖面"：摄像机移到 +X 正前方，正交看向 YZ 平面
- [ ] X 剖面下左键拖动：在 YZ 平面内平移，不旋转
- [ ] X 剖面下滚轮：拉近/远离剖面
- [ ] 点击 "3D" 切回：恢复轨道旋转行为
- [ ] 切换视角不改变滑块值
- [ ] 鼠标在滑块上拖拽时，剖面模式下不触发平移
