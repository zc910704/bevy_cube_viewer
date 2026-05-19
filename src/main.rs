mod camera;
mod cube_grid;
mod cube_material;
mod guide_line;
mod picking;
mod ui;

use bevy::prelude::*;
use bevy::input_focus::{
    tab_navigation::TabNavigationPlugin,
    InputDispatchPlugin,
};
use bevy_dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::ui_widgets::{UiWidgetsPlugins, SliderValue, SetChecked};
use bevy::ui::Checked;

use camera::{CameraState, ViewMode, orbit_camera};
use cube_grid::{CubeGrid, RangeSelectionState, SelectionMode, DIM_X, DIM_Y, DIM_Z};
use cube_material::{CubeGridMaterialPlugin, spawn_cube_grid, update_instance_data, update_hover_grid_data};
use guide_line::{ShowGuideLine, draw_guide_line};
use ui::{
    CubeGridSlider, setup_ui, update_slider_visuals, update_value_labels, on_slider_changed,
    on_view_button_changed, on_mode_button_changed, update_button_visuals,
    update_hover_coords_panel, update_hover_tooltip,
    update_cube_count, sync_range_sliders,
    FailBitCheckbox, OrbitVisibleCheckbox, GuideLineCheckbox,
    on_failbit_changed, update_checkbox_visuals,
    on_orbit_visible_changed, update_orbit_checkbox_visuals,
    on_guide_line_changed, update_guide_checkbox_visuals, update_guide_line_label,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FpsOverlayPlugin::default(),
            UiWidgetsPlugins,
            InputDispatchPlugin,
            TabNavigationPlugin,
            CubeGridMaterialPlugin,
        ))
        .init_resource::<CubeGrid>()
        .init_resource::<RangeSelectionState>()
        .init_resource::<CameraState>()
        .init_resource::<ViewMode>()
        .init_resource::<ShowGuideLine>()
        .init_resource::<picking::PickingState>()
        .add_systems(Startup, (setup_camera, spawn_cube_grid, setup_ui))
        .add_systems(Update, picking::picking_system)
        .add_systems(Update, orbit_camera)
        .add_systems(Update, on_slider_changed)
        .add_systems(Update, update_instance_data)
        .add_systems(Update, update_hover_grid_data)
        .add_systems(Update, update_slider_visuals)
        .add_systems(Update, update_value_labels)
        .add_systems(Update, on_view_button_changed)
        .add_systems(Update, on_mode_button_changed)
        .add_systems(Update, update_button_visuals)
        .add_systems(Update, update_cube_count)
        .add_systems(Update, sync_range_sliders)
        .add_systems(Update, update_hover_coords_panel)
        .add_systems(Update, update_hover_tooltip)
        .add_systems(Update, handle_esc)
        .add_observer(on_failbit_changed)
        .add_observer(on_orbit_visible_changed)
        .add_observer(on_guide_line_changed)
        .add_systems(Update, update_checkbox_visuals)
        .add_systems(Update, update_orbit_checkbox_visuals)
        .add_systems(Update, update_guide_checkbox_visuals)
        .add_systems(Update, draw_guide_line)
        .add_systems(Update, update_guide_line_label)
        .run();
}

fn setup_camera(
    mut commands: Commands,
) {
    // Isometric view from (+X, +Y, +Z) octant, far enough to see the entire array
    let iso_dir = Vec3::new(1.0, 1.0, 1.0).normalize();
    let orbit_distance = 1300.0;
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(iso_dir * orbit_distance)
            .looking_at(Vec3::ZERO, Vec3::Y),
        bevy::render::view::NoIndirectDrawing,
    ));
}

fn handle_esc(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<RangeSelectionState>,
    mut camera_state: ResMut<CameraState>,
    mut show_guide: ResMut<ShowGuideLine>,
    section_sliders: Query<Entity, (With<CubeGridSlider>, With<crate::ui::SliderAxis>)>,
    checkbox: Query<(Entity, Has<Checked>), With<FailBitCheckbox>>,
    orbit_checkbox: Query<(Entity, Has<Checked>), With<OrbitVisibleCheckbox>>,
    guide_checkbox: Query<(Entity, Has<Checked>), With<GuideLineCheckbox>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.mode {
            SelectionMode::Section => {
                state.x_slider = 0;
                state.y_slider = 0;
                state.z_slider = 0;
                state.only_failbit = false;
            }
            SelectionMode::Range => {
                state.x_min = 1;
                state.x_max = DIM_X as u32;
                state.y_min = 1;
                state.y_max = DIM_Y as u32;
                state.z_min = 1;
                state.z_max = DIM_Z as u32;
                state.only_failbit = false;
            }
        }
        camera_state.orbit_only_visible = false;
        state.dirty = true;

        // Section sliders reset to 0 (show all); range sliders synced by sync_range_sliders.
        for entity in &section_sliders {
            commands.entity(entity).insert(SliderValue(0.0));
        }

        if let Ok((entity, is_checked)) = checkbox.single() {
            if is_checked {
                commands.trigger(SetChecked {
                    entity,
                    checked: false,
                });
            }
        }

        if let Ok((entity, is_checked)) = orbit_checkbox.single() {
            if is_checked {
                commands.trigger(SetChecked {
                    entity,
                    checked: false,
                });
            }
        }

        show_guide.0 = true;
        if let Ok((entity, has_checked)) = guide_checkbox.single() {
            if !has_checked {
                commands.trigger(SetChecked {
                    entity,
                    checked: true,
                });
            }
        }
    }
}
