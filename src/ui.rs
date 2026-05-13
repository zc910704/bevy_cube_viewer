use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{
        observe, slider_self_update, CoreSliderDragState, Slider, SliderRange, SliderThumb,
        SliderValue, TrackClick,
    },
};

use crate::camera::ViewMode;
use crate::cube_grid::{compute_grid_position, CrossSectionState, DIM_X, DIM_Y, DIM_Z};
use crate::picking::PickingState;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum SliderAxis {
    X,
    Y,
    Z,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ViewButtonAxis {
    ThreeD,
    X,
    Y,
    Z,
}

#[derive(Component)]
pub struct CubeGridSlider;

#[derive(Component)]
pub struct CubeGridSliderThumb;

#[derive(Component)]
pub struct SliderValueText;

#[derive(Component)]
pub(crate) struct HoverCoordsText;

#[derive(Component)]
pub(crate) struct HoverTooltip;

const SLIDER_TRACK_COLOR: Color = Color::srgb(0.1, 0.1, 0.12);
const SLIDER_THUMB_COLOR: Color = Color::srgb(0.4, 0.7, 0.4);
const LABEL_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);
const BG_COLOR: Color = Color::srgba(0.02, 0.02, 0.04, 0.75);

const BTN_INACTIVE_COLOR: Color = Color::srgb(0.1, 0.1, 0.12);
const BTN_ACTIVE_COLOR: Color = Color::srgb(0.4, 0.7, 0.4);
const BTN_HOVER_COLOR: Color = Color::srgb(0.6, 0.85, 0.6);

pub fn setup_ui(mut commands: Commands) {
    // Build button bar content first to avoid borrow conflict.
    let button_bar_row = build_button_bar(&mut commands);

    // Button bar panel
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0 + 150.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BG_COLOR),
        ))
        .add_child(button_bar_row);

    // Slider panel
    let x_slider = build_slider(&mut commands, SliderAxis::X, DIM_X as f32, "X");
    let y_slider = build_slider(&mut commands, SliderAxis::Y, DIM_Y as f32, "Y");
    let z_slider = build_slider(&mut commands, SliderAxis::Z, DIM_Z as f32, "Z");

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BG_COLOR),
        ))
        .add_children(&[x_slider, y_slider, z_slider]);

    // Coordinate display panel (bottom-right)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            right: Val::Px(16.0),
            padding: UiRect::all(Val::Px(12.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(BG_COLOR),
    )).with_children(|parent| {
        parent.spawn((
            Text::new("--"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(LABEL_COLOR),
            HoverCoordsText,
        ));
    });

    // Floating tooltip (screen-space, follows hovered cube)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::all(Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        Visibility::Hidden,
        HoverTooltip,
    )).with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.8, 0.0)),
        ));
    });
}

fn build_button_bar(commands: &mut Commands) -> Entity {
    let row = commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(4.0),
            ..default()
        })
        .id();

    let buttons = [
        (ViewButtonAxis::ThreeD, "3D"),
        (ViewButtonAxis::X, "X 剖面"),
        (ViewButtonAxis::Y, "Y 剖面"),
        (ViewButtonAxis::Z, "Z 剖面"),
    ];

    for (axis, label) in buttons {
        let btn = commands
            .spawn((
                Button,
                Node {
                    padding: UiRect::all(Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(BTN_INACTIVE_COLOR),
                axis,
                Text::new(label),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
            ))
            .id();
        commands.entity(row).add_child(btn);
    }

    row
}

fn build_slider(commands: &mut Commands, axis: SliderAxis, max: f32, label: &str) -> Entity {
    let axis_label = commands
        .spawn((
            Text::new(format!("{} 轴", label)),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(LABEL_COLOR),
        ))
        .id();

    let value_text = commands
        .spawn((
            Text::new("0"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(LABEL_COLOR),
            axis,
            SliderValueText,
        ))
        .id();

    let label_row = commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .add_children(&[axis_label, value_text])
        .id();

    let track = commands
        .spawn((
            Node {
                height: Val::Px(8.0),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(SLIDER_TRACK_COLOR),
        ))
        .id();

    let thumb = commands
        .spawn((
            CubeGridSliderThumb,
            SliderThumb,
            Node {
                display: Display::Flex,
                width: Val::Px(14.0),
                height: Val::Px(18.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(SLIDER_THUMB_COLOR),
        ))
        .id();

    let thumb_wrapper = commands
        .spawn(Node {
            display: Display::Flex,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(14.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        })
        .add_child(thumb)
        .id();

    let slider = commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Stretch,
                height: Val::Px(18.0),
                width: Val::Percent(100.0),
                ..default()
            },
            CubeGridSlider,
            axis,
            Slider {
                track_click: TrackClick::Snap,
            },
            SliderValue(0.0),
            SliderRange::new(0.0, max),
            Hovered::default(),
            observe(slider_self_update),
        ))
        .add_children(&[track, thumb_wrapper])
        .id();

    let row = commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            width: Val::Px(560.0),
            ..default()
        })
        .add_children(&[label_row, slider])
        .id();

    row
}

/// Updates slider thumb position and highlight.
pub fn update_slider_visuals(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange, &Hovered, &CoreSliderDragState),
        (
            Or<(
                Changed<SliderValue>,
                Changed<Hovered>,
                Changed<CoreSliderDragState>,
            )>,
            With<CubeGridSlider>,
        ),
    >,
    children: Query<&Children>,
    mut thumbs: Query<
        (&mut Node, &mut BackgroundColor, Has<CubeGridSliderThumb>),
        Without<CubeGridSlider>,
    >,
) {
    for (slider_ent, value, range, hovered, drag_state) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, mut thumb_bg, is_thumb)) = thumbs.get_mut(child) {
                if is_thumb {
                    let position = range.thumb_position(value.0) * 100.0;
                    thumb_node.left = Val::Percent(position);

                    let is_active = hovered.0 | drag_state.dragging;
                    thumb_bg.0 = if is_active {
                        SLIDER_THUMB_COLOR.lighter(0.3)
                    } else {
                        SLIDER_THUMB_COLOR
                    };
                }
            }
        }
    }
}

/// Updates slider value text when slider value changes.
pub fn update_value_labels(
    sliders: Query<(&SliderValue, &SliderAxis), (Changed<SliderValue>, With<CubeGridSlider>)>,
    mut texts: Query<(&mut Text, &SliderAxis), With<SliderValueText>>,
) {
    for (value, axis) in sliders.iter() {
        for (mut text, txt_axis) in texts.iter_mut() {
            if axis == txt_axis {
                **text = format!("{:.0}", value.0);
            }
        }
    }
}

/// Syncs slider values into CrossSectionState.
pub fn on_slider_changed(
    sliders: Query<(&SliderValue, &SliderAxis), Changed<SliderValue>>,
    mut cross_section: ResMut<CrossSectionState>,
    mut ready: Local<bool>,
) {
    // Skip the first trigger — bevy_ui_widgets may fire Changed<SliderValue>
    // during initialization before the user has touched any slider.
    if !*ready {
        *ready = true;
        return;
    }
    for (value, axis) in &sliders {
        let val = value.0 as u32;
        match axis {
            SliderAxis::X => cross_section.x_slider = val,
            SliderAxis::Y => cross_section.y_slider = val,
            SliderAxis::Z => cross_section.z_slider = val,
        }
        cross_section.dirty = true;
    }
}

/// Sets ViewMode when a view button is pressed.
pub fn on_view_button_changed(
    mut interaction_query: Query<(&Interaction, &ViewButtonAxis), Changed<Interaction>>,
    mut view_mode: ResMut<ViewMode>,
) {
    for (interaction, axis) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        *view_mode = match axis {
            ViewButtonAxis::ThreeD => ViewMode::ThreeD,
            ViewButtonAxis::X => ViewMode::SectionX,
            ViewButtonAxis::Y => ViewMode::SectionY,
            ViewButtonAxis::Z => ViewMode::SectionZ,
        };
    }
}

/// Updates button background color to reflect active view mode.
pub fn update_button_visuals(
    view_mode: Res<ViewMode>,
    mut buttons: Query<(&ViewButtonAxis, &mut BackgroundColor, &Interaction)>,
) {
    for (axis, mut bg, interaction) in &mut buttons {
        let is_active = matches!(
            (*view_mode, axis),
            (ViewMode::ThreeD, ViewButtonAxis::ThreeD)
                | (ViewMode::SectionX, ViewButtonAxis::X)
                | (ViewMode::SectionY, ViewButtonAxis::Y)
                | (ViewMode::SectionZ, ViewButtonAxis::Z)
        );
        bg.0 = if is_active {
            BTN_ACTIVE_COLOR
        } else if *interaction == Interaction::Hovered {
            BTN_HOVER_COLOR
        } else {
            BTN_INACTIVE_COLOR
        };
    }
}

/// Updates the fixed coordinate panel text from PickingState.
pub(crate) fn update_hover_coords_panel(
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

/// Positions the floating tooltip in screen space above the hovered cube.
pub(crate) fn update_hover_tooltip(
    picking: Res<PickingState>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut tooltip: Query<(&mut Node, &mut Visibility, &Children), With<HoverTooltip>>,
    mut tooltip_texts: Query<&mut Text, Without<HoverTooltip>>,
) {
    let Ok((mut node, mut vis, children)) = tooltip.single_mut() else {
        return;
    };
    let (camera, cam_transform) = camera.into_inner();

    match picking.hovered_cube {
        Some((x, y, z)) => {
            let world_pos = compute_grid_position(x, y, z);
            // Offset above the cube center
            let label_pos = world_pos + Vec3::new(0.0, 0.8, 0.0);

            if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, label_pos) {
                node.left = Val::Px(screen_pos.x - 35.0);
                node.top = Val::Px(screen_pos.y - 12.0);
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }

            for child in children.iter() {
                if let Ok(mut text) = tooltip_texts.get_mut(child) {
                    **text = format!("X:{} Y:{} Z:{}", x, y, z);
                }
            }
        }
        None => {
            *vis = Visibility::Hidden;
        }
    }
}
