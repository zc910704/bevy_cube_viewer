use bevy::{
    prelude::*,
};

pub struct CubeRendererPlugin;

impl Plugin for CubeRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cubes);
    }
}

fn setup_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let blue_material = materials.add(Color::srgb_u8(124, 144, 255));

    let spacing = 1.5; // 立方体间距 1.5 (1.0 边长 + 0.5 间隙)
    let layers = 3;    // 3 层 (y = 0, 1, 2)

    for x in -2..=2 {
        for y in 0..layers {
            for z in -2..=2 {
                commands.spawn((
                    Mesh3d(cube_mesh.clone()),
                    MeshMaterial3d(blue_material.clone()),
                    Transform::from_xyz(
                        x as f32 * spacing,
                        y as f32 * spacing + 0.5,
                        z as f32 * spacing,
                    ),
                ));
            }
        }
    }
}