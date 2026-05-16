mod camera;
mod cube_grid;
mod cube_material;
mod picking;
mod ui;

use bevy::prelude::*;
use bevy::input_focus::{
    tab_navigation::TabNavigationPlugin,
    InputDispatchPlugin,
};
use bevy_dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::ui_widgets::{UiWidgetsPlugins, SliderValue};

use camera::{CameraState, ViewMode, orbit_camera};
use cube_grid::{CubeGrid, RangeSelectionState, SelectionMode, DIM_X, DIM_Y, DIM_Z};
use cube_material::{CubeGridMaterialPlugin, spawn_cube_grid, update_instance_data, update_hover_grid_data};
use ui::{
    CubeGridSlider, setup_ui, update_slider_visuals, update_value_labels, on_slider_changed,
    on_view_button_changed, on_mode_button_changed, update_button_visuals,
    update_hover_coords_panel, update_hover_tooltip,
    update_cube_count, sync_range_sliders,
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
        .run();
}

fn setup_camera(
    mut commands: Commands,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-80.0, 20.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        bevy::render::view::NoIndirectDrawing,
    ));
}

fn handle_esc(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<RangeSelectionState>,
    section_sliders: Query<Entity, (With<CubeGridSlider>, With<crate::ui::SliderAxis>)>,
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
                state.x_min = 1;
                state.x_max = DIM_X as u32;
                state.y_min = 1;
                state.y_max = DIM_Y as u32;
                state.z_min = 1;
                state.z_max = DIM_Z as u32;
            }
        }
        state.dirty = true;

        // Section sliders reset to 0 (show all); range sliders synced by sync_range_sliders.
        for entity in &section_sliders {
            commands.entity(entity).insert(SliderValue(0.0));
        }
    }
}
