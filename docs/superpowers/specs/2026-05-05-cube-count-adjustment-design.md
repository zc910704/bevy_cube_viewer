# 动态调整立方体数量 - 设计文档

## 概述

通过键盘输入动态调整 3D 立方体阵列的 X/Y/Z 维度数量，使用 `FpsOverlayPlugin` 显示帧率。

## 按键映射

| 按键 | 动作 |
|------|------|
| `X` / `Shift+X` | X 轴立方体数量 -1 / +1 |
| `Y` / `Shift+Y` | Y 轴立方体数量 -1 / +1 |
| `Z` / `Shift+Z` | Z 轴立方体数量 -1 / +1 |

- 每个维度最少为 1
- Y 轴从 0 开始（地面层）
- X/Z 以 0 为中心对称分布

## 架构设计

### SharedState

```rust
#[derive(Resource, Clone)]
pub struct SharedState {
    pub selected: Arc<Mutex<Option<(i32, i32, i32)>>>,
    pub requested_xyz: Arc<Mutex<(u32, u32, u32)>>,  // 目标 XYZ 数量
}
```

### 增量更新策略

不采用全量重新生成，而是：

1. **跟踪现有立方体**：为每个立方体附加 `CubePos` 组件标记其目标位置 `(x, y, z)`
2. **计算差异**：比较当前世界中的立方体与目标阵列
3. **最小化操作**：
   - 多余的立方体：despawn
   - 缺少的立方体：spawn
   - 位置变化的立方体：transform 更新

### 系统设计

1. **InputSystem**：读取键盘输入，更新 `requested_xyz`
2. **CubeUpdateSystem**：
   - 检测 `requested_xyz` 变化（`ChangedRes`）
   - 计算目标位置集合
   - 增删或移动立方体实体
   - 使用 `SpatialQuery` 或标记组件辅助查找

### 组件

```rust
#[derive(Component)]
struct CubePos {
    x: i32, y: i32, z: i32,  // 目标位置（用于去重）
}
```

## 视觉反馈

- `FpsOverlayPlugin` 显示帧率
- 无需显示 XYZ 数值

## 依赖

- `bevy` (本地路径)
- `bevy_diagnostic::FpsOverlayPlugin`
