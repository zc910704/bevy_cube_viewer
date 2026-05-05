# 动态调整立方体数量 - 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 通过键盘 X/Y/Z 动态调整立方体阵列维度，使用 FpsOverlayPlugin 显示帧率，增量更新最小化性能影响。

**Architecture:** 使用 `CubePos` 组件标记立方体位置，通过 `ChangedRes<SharedState>` 检测变化并增量增删立方体，而非全量重新生成。

**Tech Stack:** Bevy ECS, bevy_diagnostic::FpsOverlayPlugin

---

## 文件结构

- **Modify:** `src/shared_state.rs` - 添加 `CubePos` 组件
- **Modify:** `src/cube_renderer.rs` - 改为可复用系统，添加增量更新逻辑
- **Modify:** `src/main.rs` - 添加键盘输入系统和 FpsOverlayPlugin

---

## Task 1: 添加 CubePos 组件

**Files:**
- Modify: `src/shared_state.rs`

- [ ] **Step 1: 添加 CubePos 组件定义**

在 `shared_state.rs` 末尾添加：

```rust
use bevy::prelude::Component;

#[derive(Component)]
pub struct CubePos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
```

- [ ] **Step 2: 提交**

```bash
git add src/shared_state.rs
git commit -m "feat: 添加 CubePos 组件用于标记立方体位置"
```

---

## Task 2: 修改 cube_renderer.rs 实现增量更新

**Files:**
- Modify: `src/cube_renderer.rs`

- [ ] **Step 1: 修改 CubeRendererPlugin，移除 Startup 系统**

将 `src/cube_renderer.rs` 改为：

```rust
use bevy::{
    prelude::*,
};
use crate::shared_state::CubePos;

pub struct CubeRendererPlugin;

impl Plugin for CubeRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_cubes_system);
    }
}

fn spawn_cubes_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    shared_state: Res<crate::shared_state::SharedState>,
) {
    let requested_xyz = *shared_state.requested_xyz.lock().unwrap();
    let (target_x, target_y, target_z) = requested_xyz;

    // 获取当前已存在的立方体位置
    let existing_cubes: Vec<(i32, i32, i32)> = CubePos::iter_in_world().map(|e| {
        let pos = e.get::<CubePos>().unwrap();
        (pos.x, pos.y, pos.z)
    }).collect();

    // 计算目标位置集合
    let mut target_positions: Vec<(i32, i32, i32)> = Vec::new();
    let spacing = 1.5;

    for x in 0..target_x as i32 {
        for y in 0..target_y as i32 {
            for z in 0..target_z as i32 {
                target_positions.push((x, y, z));
            }
        }
    }

    // 找出需要删除的立方体（存在但不在目标中）
    for pos in &existing_cubes {
        if !target_positions.contains(pos) {
            // despawn 对应的实体
            for entity in CubePos::iter_with_entity_in_world() {
                if entity.get::<CubePos>().unwrap().x == pos.0
                    && entity.get::<CubePos>().unwrap().y == pos.1
                    && entity.get::<CubePos>().unwrap().z == pos.2
                {
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }
    }

    // 找出需要添加的立方体（目标中但不存在）
    for target_pos in &target_positions {
        if !existing_cubes.contains(target_pos) {
            let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
            let blue_material = materials.add(Color::srgb_u8(124, 144, 255));

            commands.spawn((
                Mesh3d(cube_mesh),
                MeshMaterial3d(blue_material),
                Transform::from_xyz(
                    target_pos.0 as f32 * spacing,
                    target_pos.1 as f32 * spacing + 0.5,
                    target_pos.2 as f32 * spacing,
                ),
                CubePos {
                    x: target_pos.0,
                    y: target_pos.1,
                    z: target_pos.2,
                },
            ));
        }
    }
}
```

- [ ] **Step 2: 提交**

```bash
git add src/cube_renderer.rs
git commit -m "feat: 重写 CubeRendererPlugin 为增量更新系统"
```

---

## Task 3: 修改 main.rs 添加键盘输入和 FPS 显示

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 添加键盘输入系统**

在 `main.rs` 中添加：

```rust
fn adjust_cubes(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut shared_state: ResMut<SharedState>,
) {
    let mut xyz = *shared_state.requested_xyz.lock().unwrap();
    let (mut x, mut y, mut z) = xyz;

    if keyboard.just_pressed(KeyCode::KeyX) && !keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        x = x.saturating_sub(1).max(1);
    }
    if keyboard.just_pressed(KeyCode::ShiftX) || (keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) && keyboard.just_pressed(KeyCode::KeyX)) {
        x = x.saturating_add(1);
    }

    // 类似处理 Y 和 Z...

    shared_state.requested_xyz = Arc::new(Mutex::new((x, y, z)));
}
```

注意：Bevy 的 `just_pressed` 配合 Shift 检测需要特殊处理，因为 Shift+X 会被操作系统识别为另一个键。可以使用 `ShiftX` 虚拟键或改用按住 Shift 时的替代方案。

建议的简化方案：

```rust
fn adjust_cubes(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut shared_state: ResMut<SharedState>,
) {
    let mut xyz = *shared_state.requested_xyz.lock().unwrap();
    let (mut x, mut y, mut z) = xyz;

    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if keyboard.just_pressed(KeyCode::KeyX) {
        if shift { x = x.saturating_add(1); } else { x = x.saturating_sub(1).max(1); }
    }
    if keyboard.just_pressed(KeyCode::KeyY) {
        if shift { y = y.saturating_add(1); } else { y = y.saturating_sub(1).max(1); }
    }
    if keyboard.just_pressed(KeyCode::KeyZ) {
        if shift { z = z.saturating_add(1); } else { z = z.saturating_sub(1).max(1); }
    }

    *shared_state.requested_xyz.lock().unwrap() = (x, y, z);
}
```

- [ ] **Step 2: 更新 App builder 添加新系统和插件**

```rust
use bevy_diagnostic::FpsOverlayPlugin;

App::new()
    .insert_resource(shared_state)
    .init_resource::<CameraState>()
    .add_plugins((
        DefaultPlugins,
        FpsOverlayPlugin::default(),
        CubeRendererPlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (
        orbit_camera,
        adjust_cubes,  // 新增
    ))
    .run();
```

- [ ] **Step 3: 提交**

```bash
git add src/main.rs
git commit -m "feat: 添加键盘调整立方体数量和 FPS 显示"
```

---

## Task 4: 测试验证

- [ ] **Step 1: 运行程序**

```bash
cargo run
```

- [ ] **Step 2: 验证功能**

1. 检查 FPS 是否显示在屏幕上
2. 按 X/X+Shift 观察 X 轴立方体数量变化
3. 按 Y/Y+Shift 观察 Y 轴立方体数量变化
4. 按 Z/Z+Shift 观察 Z 轴立方体数量变化
5. 验证最小值为 1
6. 验证增量更新（观察立方体增删而非全量重绘）

- [ ] **Step 3: 提交最终更改**

```bash
git add -A
git commit -m "feat: 完成动态调整立方体数量功能"
```
