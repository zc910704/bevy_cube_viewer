# View Switching Design

## Summary

Add 4 view mode buttons (3D, X Section, Y Section, Z Section) to switch camera behavior. 3D is the current orbit camera. Section views fix the camera orthogonal to a section plane with pan/zoom controls.

## Requirements

- 4 buttons: "3D", "X 剖面", "Y 剖面", "Z 剖面"
- Buttons placed in an independent panel above the existing slider panel (bottom-left area)
- Active button highlighted with slider thumb green color
- 3D mode: current orbit camera (left-drag rotate, scroll zoom)
- Section mode: camera faces the section plane from the positive axis direction; left-drag pans parallel to the plane; scroll changes distance to the plane
- View switching is pure camera behavior — does not modify cross-section slider values
- On section enter: auto-calculate distance to fit the visible extent in view
- On 3D return: keep current camera position, infer orbit parameters from transform

## New Types

### ViewMode (camera.rs)

```rust
#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    ThreeD,
    SectionX,
    SectionY,
    SectionZ,
}
```

### CameraState extension (camera.rs)

```rust
pub struct CameraState {
    pub orbit_distance: f32,        // existing
    pub section_distance: f32,      // new: camera-to-plane distance
    pub section_target: Vec3,       // new: look-at point for pan offset
}
```

## Camera Behavior Per Mode

### ThreeD
Unchanged from current: left-drag rotates (yaw/pitch), scroll changes orbit_distance.

### SectionX (camera at +X, looking toward -X)
- Rotation: `Quat::from_rotation_y(-PI/2)`
- Screen horizontal = world Z (grid Y depth), Screen vertical = world Y (grid Z up/down)
- Left-drag: pan section_target in YZ (world) plane
- Scroll: adjust section_distance, clamped to [10, 2000]

### SectionY (camera at +Z world, looking toward -Z)
- Rotation: `Quat::IDENTITY` looking along -Z
- Left-drag: pan section_target in XY (world) plane
- Scroll: same as above

### SectionZ (camera at +Y world, looking toward -Y)
- Rotation: `Quat::from_rotation_x(PI/2)` looking toward -Y
- Left-drag: pan section_target in XZ (world) plane
- Scroll: same as above

## Auto-Fit Distance

On entering a section mode, compute:

```rust
fn fit_distance(fov: f32, visible_extent: f32) -> f32 {
    (visible_extent / 2.0) / (fov / 2.0).tan()
}
```

- X section: visible_extent = max(DIM_Y * spacing, DIM_Z * spacing)
- Y section: visible_extent = max(DIM_X * spacing, DIM_Z * spacing)
- Z section: visible_extent = max(DIM_X * spacing, DIM_Y * spacing)

## UI Design

Independent button bar panel above the existing slider panel.

### Layout
Both panels positioned at bottom-left, stacked vertically with a gap.

### Button States
- Inactive: dark gray background (track color)
- Active: green background (slider thumb color `Color::srgb(0.4, 0.7, 0.4)`)
- Hover: slightly lighter

### Interaction
- Standard Bevy `Button` + `Interaction` component queries
- `on_view_button_changed` system writes to `ViewMode` Resource
- Uses `Changed<Interaction>` per-button to detect clicks

## Data Flow

```
UI button click
  → on_view_button_changed system → writes ViewMode Resource
  → ViewMode change detected in orbit_camera
  → if 3D→Section: set fixed rotation, compute section_distance, reset section_target
  → if Section→3D: keep transform, compute orbit_distance from camera.translation
  → per-frame: branch on ViewMode to handle input appropriately
```

## Files Changed

| File | Changes |
|------|---------|
| `src/camera.rs` | New `ViewMode` enum, extend `CameraState`, branch `orbit_camera` |
| `src/ui.rs` | New button bar panel, button component, `on_view_button_changed` system |
| `src/main.rs` | Register `ViewMode` Resource, register `on_view_button_changed` system |

## Exclusions

- No slider coupling: view buttons do not modify `CrossSectionState`
- No transition animation: camera snaps on mode switch
- No keyboard shortcuts for view switching (buttons only)
