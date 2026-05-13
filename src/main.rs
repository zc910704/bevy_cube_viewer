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
use cube_grid::{CubeGrid, CrossSectionState};
use cube_material::{CubeGridMaterialPlugin, spawn_cube_grid, update_instance_data, update_hover_grid_data};
use ui::{
    CubeGridSlider, setup_ui, update_slider_visuals, update_value_labels, on_slider_changed,
    on_view_button_changed, update_button_visuals, update_hover_coords_panel,
    update_hover_tooltip,
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
        .init_resource::<CrossSectionState>()
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
        .add_systems(Update, update_button_visuals)
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
    mut cross_section: ResMut<CrossSectionState>,
    slider_query: Query<Entity, With<CubeGridSlider>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        cross_section.x_slider = 0;
        cross_section.y_slider = 0;
        cross_section.z_slider = 0;
        cross_section.dirty = true;

        for entity in &slider_query {
            commands.entity(entity).insert(SliderValue(0.0));
        }
    }
}
