use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{
        observe, slider_self_update, CoreSliderDragState, Slider, SliderRange, SliderThumb,
        SliderValue, TrackClick,
    },
};

use crate::cube_grid::{CrossSectionState, DIM_X, DIM_Y, DIM_Z};

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum SliderAxis {
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

const SLIDER_TRACK_COLOR: Color = Color::srgb(0.1, 0.1, 0.12);
const SLIDER_THUMB_COLOR: Color = Color::srgb(0.4, 0.7, 0.4);
const LABEL_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);
const BG_COLOR: Color = Color::srgba(0.02, 0.02, 0.04, 0.75);

pub fn setup_ui(mut commands: Commands) {
    let panel = commands
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
        .id();

    let x_slider = build_slider(&mut commands, SliderAxis::X, DIM_X as f32, "X");
    let y_slider = build_slider(&mut commands, SliderAxis::Y, DIM_Y as f32, "Y");
    let z_slider = build_slider(&mut commands, SliderAxis::Z, DIM_Z as f32, "Z");

    commands.entity(panel).add_children(&[x_slider, y_slider, z_slider]);
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
            width: Val::Px(280.0),
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
