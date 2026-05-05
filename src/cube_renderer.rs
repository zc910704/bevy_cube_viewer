use bevy::{
    prelude::*,
};
use std::collections::HashSet;
use crate::shared_state::{SharedState, CubePos};

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
    shared_state: Res<SharedState>,
    query: Query<(Entity, &CubePos)>,
) {
    let (target_x, target_y, target_z) = *shared_state.requested_xyz.lock().unwrap();

    // 预创建 mesh 和 material（避免每帧重复创建）
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let blue_material = materials.add(Color::srgb_u8(124, 144, 255));

    // 收集现有立方体位置到 HashSet
    let existing: HashSet<(i32, i32, i32)> = query.iter()
        .map(|(_, pos)| (pos.x, pos.y, pos.z))
        .collect();

    // 目标位置集合
    let target_positions: HashSet<(i32, i32, i32)> = (0..target_x as i32)
        .flat_map(|x| {
            let x_val = x;
            (0..target_y as i32)
                .flat_map(move |y| {
                    let y_val = y;
                    (0..target_z as i32)
                        .map(move |z| (x_val, y_val, z))
                })
        })
        .collect();

    // 删除多余立方体（存在但不在目标中）
    for (entity, cube_pos) in query.iter() {
        if !target_positions.contains(&(cube_pos.x, cube_pos.y, cube_pos.z)) {
            commands.entity(entity).despawn();
        }
    }

    // 添加缺失立方体（目标中但不存在）
    let spacing = 1.5;
    for target_pos in &target_positions {
        if !existing.contains(target_pos) {
            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(blue_material.clone()),
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
