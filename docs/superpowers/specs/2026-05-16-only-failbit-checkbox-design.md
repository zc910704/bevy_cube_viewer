# Only FailBit Checkbox Design

## Overview

在按钮栏 Range Mode 按钮右边添加一个 "Only FailBit" checkbox。勾选后，只显示值为 1（红色）的正方体，过滤掉值为 0（白色）和 2（灰色）的方块。checkbox 始终可见，Section 和 Range 两种模式下都生效。

## Data Layer (`cube_grid.rs`)

`RangeSelectionState` 新增字段：

```rust
pub only_failbit: bool,  // default: false
```

`compute_visible_instances` 在坐标循环内增加过滤：

```rust
if state.only_failbit && grid.get(x, y, z) != 1 {
    continue;
}
```

## UI Layer (`ui.rs`)

### Checkbox 组件

`build_button_bar` 中，在 Range Mode 按钮右边插入 checkbox：

- 组件：`Checkbox` + `Checkable` + `Node` + `BackgroundColor`
- 标签文本 "Only FailBit" 放在 checkbox 右侧
- `observe(checkbox_self_update)` 自动管理 `Checked` 状态
- 新增标记组件 `FailBitCheckbox`

需要引入的 API：

| 来源 | 符号 |
|------|------|
| `bevy_ui` | `Checkable` |
| `bevy_ui_widgets` | `Checkbox`, `ValueChange`, `checkbox_self_update`, `SetChecked` |

### Cube Count 适配

`update_cube_count` 需新增 `Res<CubeGrid>` 参数。当 `only_failbit` 为 true 时，Section 模式下需遍历网格统计红色方块数（而非简单乘法）；Range 模式下在已有的循环中也可以顺便统计。

### 事件处理

新增系统 `on_failbit_changed`：

- 监听 `Checkbox` 实体上的 `ValueChange<bool>` 事件
- 写入 `RangeSelectionState.only_failbit`，设置 `dirty = true`

## ESC Reset (`main.rs`)

`handle_esc` 中同时重置 `only_failbit = false`。同步 checkbox UI 状态时，需要先通过 `Query<(Entity, Has<Checked>), With<FailBitCheckbox>>` 获取 checkbox 实体和当前状态，仅当状态不一致时调用 `commands.trigger(SetChecked { entity, checked: false })`。

## Files Changed

| File | Change |
|------|--------|
| `cube_grid.rs` | `RangeSelectionState` 加 `only_failbit` 字段；`compute_visible_instances` 加过滤 |
| `ui.rs` | 新增 `FailBitCheckbox` 组件；`build_button_bar` 加 checkbox UI；`update_cube_count` 适配 `only_failbit` 计数；新增 `on_failbit_changed` 系统 |
| `main.rs` | 注册 `on_failbit_changed` 系统；`handle_esc` 重置 `only_failbit` 并同步 checkbox |
