# 鼠标悬停拾取正方体坐标 — 设计文档

## 目标

鼠标悬停时显示正方体的网格坐标 (grid_x, grid_y, grid_z)，并高亮该正方体。坐标同时显示在固定 UI 面板（右下角）和浮动标签（跟随正方体）。

## 交互规格

| 行为 | 描述 |
|------|------|
| 悬停 | 鼠标移动到正方体上方时，实时高亮并显示坐标 |
| 离开 | 鼠标不在任何正方体上时，固定面板显示 `--`，浮动标签隐藏 |
| 高亮 | 悬停正方体变为亮黄色，在实例数据中修改其颜色 |
| 坐标系统 | 显示网格索引坐标（X:0~63, Y:0~1023, Z:0~15） |
| 横截面约束 | 仅对当前可见的正方体进行拾取检测 |

## 核心算法：3D DDA 射线遍历

### 射线构建

```
1. 获取鼠标屏幕坐标 (cursor_x, cursor_y)
2. 用相机 view-projection 逆矩阵反投影 → 世界空间射线原点 + 方向
```

Bevy 提供 `Camera::viewport_to_world()` 或手动通过 `Transform` + `Projection` 的逆运算得到射线。

### DDA 步进遍历

使用 Amanatides-Woo 算法在 3D 均匀网格中逐格子遍历：

```
1. 计算射线进入/离开网格 AABB 的 t 值 (t_min, t_max)
2. 如果不相交 → 无命中，返回 None
3. 确定射线起点所在的网格坐标 (grid_x, grid_y, grid_z)
4. 对每个轴计算步进方向和 t 增量：
   - step_dir = ray_dir.sign()
   - t_delta = CUBE_SPACING / ray_dir.abs()
   - t_max_per_axis = 到下一个格子边界的 t 值
5. 循环：
   a. 检查当前格子是否在可见范围内（受横截面 slider 约束）
   b. 在范围内 → 停止，返回 (grid_x, grid_y, grid_z)
   c. 选择 t_max 最小的轴，前进一步，更新该轴 t_max
   d. 如果走出网格范围 → 返回 None
```

### 性能

- 最坏情况遍历：对角线穿越 64×1024×16 网格 ≈ 1100 步
- 正常视角：通常 200-500 步
- 每步是简单的整数运算 + 边界检查
- 每帧执行一次，远在帧预算之内

### 可见性判断

步进时检查当前格子是否被横截面滑块过滤：
- `x_slider != 0 && grid_x != (x_slider - 1)` → 跳过
- `y_slider != 0 && grid_y != (y_slider - 1)` → 跳过
- `z_slider != 0 && grid_z != (z_slider - 1)` → 跳过

## 模块设计

### 新增：`src/picking.rs`

```rust
#[derive(Resource, Default)]
pub struct PickingState {
    /// 当前悬停的网格坐标，无悬停时为 None
    pub hovered_cube: Option<(usize, usize, usize)>,
}

/// 执行射线-DDA 拾取，更新 PickingState
pub fn picking_system(
    camera: Single<(&Camera, &GlobalTransform, &Projection)>,
    windows: Query<&Window>,
    cross_section: Res<CrossSectionState>,
    mut picking: ResMut<PickingState>,
) {
    // 1. 获取鼠标屏幕坐标
    // 2. 构建世界空间射线
    // 3. DDA 遍历网格
    // 4. 写入 picking.hovered_cube
}
```

### 改动：`src/cube_material.rs`

`update_instance_data` 中增加高亮逻辑：

```rust
pub fn update_instance_data(
    grid: Res<CubeGrid>,
    cross_section: Res<CrossSectionState>,
    picking: Res<PickingState>,
    mut query: Query<&mut InstanceMaterialData>,
) {
    if !cross_section.is_changed() && !grid.is_changed() && !picking.is_changed() {
        return;
    }
    let mut visible = compute_visible_instances(&grid, &cross_section);
    // 如果当前有悬停目标，替换其颜色为高亮色
    if let Some((hx, hy, hz)) = picking.hovered_cube {
        // 计算悬停实例在 visible 向量中的索引
        // 需要与 compute_visible_instances 的遍历顺序一致
        if let Some(inst) = find_instance_mut(&mut visible, hx, hy, hz, &cross_section) {
            inst.color = HIGHLIGHT_COLOR;
        }
    }
    for mut data in &mut query {
        data.0 = visible.clone();
    }
}
```

高亮色的常量定义：
```rust
const HIGHLIGHT_COLOR: [f32; 4] = [1.0, 0.8, 0.0, 1.0]; // 亮黄色
```

### 改动：`src/ui.rs`

**新增固定坐标面板**：在 `setup_ui` 中创建右下方面板，初始文本为 `--`。

```rust
#[derive(Component)]
struct HoverCoordsText;

// 更新系统：
fn update_hover_coords_panel(
    picking: Res<PickingState>,
    mut texts: Query<&mut Text, With<HoverCoordsText>>,
) {
    for mut text in &mut texts {
        **text = match picking.hovered_cube {
            Some((x, y, z)) => format!("X:{}  Y:{}  Z:{}", x, y, z),
            None => "--".into(),
        };
    }
}
```

**新增浮动标签**：一个绝对定位的 UI 元素，每帧根据悬停正方体的世界坐标投影到屏幕位置。

```rust
#[derive(Component)]
struct HoverTooltip;

fn update_hover_tooltip(
    picking: Res<PickingState>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut tooltip: Query<(&mut Node, &mut Visibility), With<HoverTooltip>>,
) {
    if let Some((x, y, z)) = picking.hovered_cube {
        let world_pos = compute_grid_position(x, y, z);
        // 投影到屏幕空间
        if let Some(screen_pos) = camera.world_to_viewport(..., world_pos) {
            tooltip.node.left = Val::Px(screen_pos.x);
            tooltip.node.top = Val::Px(screen_pos.y);
            tooltip.visibility = Visibility::Visible;
        }
    } else {
        tooltip.visibility = Visibility::Hidden;
    }
}
```

### 改动：`src/main.rs`

```rust
mod picking;

// 新增资源
.init_resource::<PickingState>()

// 新增系统
.add_systems(Update, (
    picking::picking_system,
    ui::update_hover_coords_panel,
    ui::update_hover_tooltip,
))
```

## 数据流

```
鼠标位置 → picking_system (DDA 遍历)
              │
              ▼
         PickingState { hovered_cube }
              │
    ┌─────────┼─────────────┐
    ▼         ▼             ▼
update_    update_       update_
instance_  hover_coords_ hover_
data       panel         tooltip
    │         │             │
    ▼         ▼             ▼
GPU Buffer  Text 更新   Node 定位
(高亮色)
```

## 不做的

- 点击选中/钉住坐标（目前只做悬停）
- 世界坐标显示（目前只显示网格坐标）
- 多选
