use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{
        observe, slider_self_update, CoreSliderDragState, Slider, SliderRange, SliderThumb,
        SliderValue, TrackClick,
    },
};

use crate::camera::ViewMode;
use crate::cube_grid::{
    compute_grid_position, RangeSelectionState, SelectionMode, DIM_X, DIM_Y, DIM_Z,
};
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

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum RangeSliderAxis {
    XMin,
    XMax,
    YMin,
    YMax,
    ZMin,
    ZMax,
}

#[derive(Component)]
pub struct ModeButton;

#[derive(Component)]
pub struct SectionSliderPanel;

#[derive(Component)]
pub struct RangeSliderPanel;

#[derive(Component)]
pub struct CubeCountText;

#[derive(Component)]
pub(crate) struct ButtonBarPanel;

#[derive(Component)]
pub(crate) struct HoverTooltip;

const SECTION_PANEL_HEIGHT: f32 = 150.0;
const RANGE_PANEL_HEIGHT: f32 = 310.0;

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
                bottom: Val::Px(16.0 + SECTION_PANEL_HEIGHT),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BG_COLOR),
            ButtonBarPanel,
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
            Visibility::Visible,
            SectionSliderPanel,
        ))
        .add_children(&[x_slider, y_slider, z_slider]);

    // Range slider panel (hidden by default)
    let x_min = build_range_slider(
        &mut commands,
        RangeSliderAxis::XMin,
        1.0,
        DIM_X as f32,
        1.0,
        "min",
    );
    let x_max = build_range_slider(
        &mut commands,
        RangeSliderAxis::XMax,
        1.0,
        DIM_X as f32,
        DIM_X as f32,
        "max",
    );
    let y_min = build_range_slider(
        &mut commands,
        RangeSliderAxis::YMin,
        1.0,
        DIM_Y as f32,
        1.0,
        "min",
    );
    let y_max = build_range_slider(
        &mut commands,
        RangeSliderAxis::YMax,
        1.0,
        DIM_Y as f32,
        DIM_Y as f32,
        "max",
    );
    let z_min = build_range_slider(
        &mut commands,
        RangeSliderAxis::ZMin,
        1.0,
        DIM_Z as f32,
        1.0,
        "min",
    );
    let z_max = build_range_slider(
        &mut commands,
        RangeSliderAxis::ZMax,
        1.0,
        DIM_Z as f32,
        DIM_Z as f32,
        "max",
    );

    let x_label = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            ..default()
        })
        .with_child((
            Text::new("X Axis"),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.53, 0.76, 0.91)),
        ))
        .id();
    let y_label = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            ..default()
        })
        .with_child((
            Text::new("Y Axis"),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.53, 0.76, 0.91)),
        ))
        .id();
    let z_label = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            ..default()
        })
        .with_child((
            Text::new("Z Axis"),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.53, 0.76, 0.91)),
        ))
        .id();
    let count_text = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            ..default()
        })
        .with_child((
            Text::new("Showing -- cubes"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.4, 0.7, 0.4)),
            CubeCountText,
        ))
        .id();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BG_COLOR),
            Visibility::Hidden,
            RangeSliderPanel,
        ))
        .add_children(&[
            x_label, x_min, x_max, y_label, y_min, y_max, z_label, z_min, z_max, count_text,
        ]);

    // Coordinate display panel (bottom-right)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                right: Val::Px(16.0),
                padding: UiRect::all(Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BG_COLOR),
        ))
        .with_children(|parent| {
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
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            Visibility::Hidden,
            HoverTooltip,
        ))
        .with_children(|parent| {
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
        (ViewButtonAxis::X, "X Section"),
        (ViewButtonAxis::Y, "Y Section"),
        (ViewButtonAxis::Z, "Z Section"),
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

    let mode_btn = commands
        .spawn((
            Button,
            Node {
                padding: UiRect::all(Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(BTN_INACTIVE_COLOR),
            ModeButton,
            Text::new("Range Mode"),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.85, 0.85)),
        ))
        .id();
    commands.entity(row).add_child(mode_btn);

    row
}

fn build_slider(commands: &mut Commands, axis: SliderAxis, max: f32, label: &str) -> Entity {
    let axis_label = commands
        .spawn((
            Text::new(format!("{} Axis", label)),
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

fn build_range_slider(
    commands: &mut Commands,
    axis: RangeSliderAxis,
    range_start: f32,
    range_end: f32,
    default_val: f32,
    label: &str,
) -> Entity {
    let axis_label = commands
        .spawn((
            Text::new(label.to_string()),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(LABEL_COLOR),
        ))
        .id();

    let value_text = commands
        .spawn((
            Text::new(format!("{:.0}", default_val)),
            TextFont {
                font_size: 11.0,
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
                height: Val::Px(6.0),
                border_radius: BorderRadius::all(Val::Px(3.0)),
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
                width: Val::Px(12.0),
                height: Val::Px(16.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                border_radius: BorderRadius::all(Val::Px(3.0)),
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
            right: Val::Px(12.0),
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
                height: Val::Px(16.0),
                width: Val::Percent(100.0),
                ..default()
            },
            CubeGridSlider,
            axis,
            Slider {
                track_click: TrackClick::Snap,
            },
            SliderValue(default_val),
            SliderRange::new(range_start, range_end),
            Hovered::default(),
            observe(slider_self_update),
        ))
        .add_children(&[track, thumb_wrapper])
        .id();

    commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(1.0),
            width: Val::Px(560.0),
            ..default()
        })
        .add_children(&[label_row, slider])
        .id()
}

/// Updates slider thumb position and highlight.
pub fn update_slider_visuals(
    sliders: Query<
        (
            Entity,
            &SliderValue,
            &SliderRange,
            &Hovered,
            &CoreSliderDragState,
        ),
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
    sliders: Query<
        (&SliderValue, Option<&SliderAxis>, Option<&RangeSliderAxis>),
        (Changed<SliderValue>, With<CubeGridSlider>),
    >,
    mut texts: Query<
        (&mut Text, Option<&SliderAxis>, Option<&RangeSliderAxis>),
        With<SliderValueText>,
    >,
) {
    for (value, axis, range_axis) in sliders.iter() {
        if let Some(axis) = axis {
            for (mut text, txt_axis, _) in texts.iter_mut() {
                if let Some(txt_axis) = txt_axis {
                    if axis == txt_axis {
                        **text = format!("{:.0}", value.0);
                    }
                }
            }
        }
        if let Some(axis) = range_axis {
            for (mut text, _, txt_axis) in texts.iter_mut() {
                if let Some(txt_axis) = txt_axis {
                    if axis == txt_axis {
                        **text = format!("{:.0}", value.0);
                    }
                }
            }
        }
    }
}

/// Syncs slider values into RangeSelectionState.
/// Only writes when the u32 value actually changes, to avoid triggering
/// expensive instance-buffer rebuilds on every drag frame.
pub fn on_slider_changed(
    sliders: Query<
        (&SliderValue, Option<&SliderAxis>, Option<&RangeSliderAxis>),
        Changed<SliderValue>,
    >,
    mut state: ResMut<RangeSelectionState>,
    mut ready: Local<bool>,
) {
    if !*ready {
        *ready = true;
        return;
    }

    for (value, axis, range_axis) in &sliders {
        let val = value.0 as u32;
        if let Some(axis) = axis {
            let changed = match axis {
                SliderAxis::X => {
                    if state.x_slider != val {
                        state.x_slider = val;
                        true
                    } else {
                        false
                    }
                }
                SliderAxis::Y => {
                    if state.y_slider != val {
                        state.y_slider = val;
                        true
                    } else {
                        false
                    }
                }
                SliderAxis::Z => {
                    if state.z_slider != val {
                        state.z_slider = val;
                        true
                    } else {
                        false
                    }
                }
            };
            if changed {
                state.dirty = true;
            }
        }
        if let Some(axis) = range_axis {
            let changed = match axis {
                RangeSliderAxis::XMin => {
                    if state.x_min != val {
                        state.x_min = val;
                        if val > state.x_max {
                            state.x_max = val;
                        }
                        true
                    } else {
                        false
                    }
                }
                RangeSliderAxis::XMax => {
                    if state.x_max != val {
                        state.x_max = val;
                        if val < state.x_min {
                            state.x_min = val;
                        }
                        true
                    } else {
                        false
                    }
                }
                RangeSliderAxis::YMin => {
                    if state.y_min != val {
                        state.y_min = val;
                        if val > state.y_max {
                            state.y_max = val;
                        }
                        true
                    } else {
                        false
                    }
                }
                RangeSliderAxis::YMax => {
                    if state.y_max != val {
                        state.y_max = val;
                        if val < state.y_min {
                            state.y_min = val;
                        }
                        true
                    } else {
                        false
                    }
                }
                RangeSliderAxis::ZMin => {
                    if state.z_min != val {
                        state.z_min = val;
                        if val > state.z_max {
                            state.z_max = val;
                        }
                        true
                    } else {
                        false
                    }
                }
                RangeSliderAxis::ZMax => {
                    if state.z_max != val {
                        state.z_max = val;
                        if val < state.z_min {
                            state.z_min = val;
                        }
                        true
                    } else {
                        false
                    }
                }
            };
            if changed {
                state.dirty = true;
            }
        }
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

/// Toggles between Section and Range modes when mode button is pressed.
pub fn on_mode_button_changed(
    mut interaction_query: Query<(&Interaction, &ModeButton), Changed<Interaction>>,
    mut state: ResMut<RangeSelectionState>,
    mut panel_visibility: Query<
        (&mut Visibility, Has<SectionSliderPanel>),
        Or<(With<SectionSliderPanel>, With<RangeSliderPanel>)>,
    >,
    mut mode_btns: Query<(&ModeButton, &mut Text), With<Button>>,
    mut button_bar: Query<&mut Node, With<ButtonBarPanel>>,
) {
    for (interaction, _) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let new_mode = match state.mode {
            SelectionMode::Section => SelectionMode::Range,
            SelectionMode::Range => SelectionMode::Section,
        };
        state.mode = new_mode;
        state.dirty = true;

        let is_section = new_mode == SelectionMode::Section;
        for (mut vis, is_section_panel) in &mut panel_visibility {
            *vis = if is_section_panel {
                if is_section {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }
            } else {
                if is_section {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                }
            };
        }

        for mut node in &mut button_bar {
            node.bottom = Val::Px(
                16.0 + if is_section {
                    SECTION_PANEL_HEIGHT
                } else {
                    RANGE_PANEL_HEIGHT
                },
            );
        }

        for (_, mut text) in &mut mode_btns {
            **text = if is_section {
                "Range Mode".into()
            } else {
                "Section Mode".into()
            };
        }
    }
}

/// Updates the cube count text when state changes.
pub fn update_cube_count(
    state: Res<RangeSelectionState>,
    mut texts: Query<&mut Text, With<CubeCountText>>,
) {
    if !state.is_changed() {
        return;
    }
    for mut text in &mut texts {
        let count = match state.mode {
            SelectionMode::Section => {
                let xc = if state.x_slider == 0 { DIM_X } else { 1 };
                let yc = if state.y_slider == 0 { DIM_Y } else { 1 };
                let zc = if state.z_slider == 0 { DIM_Z } else { 1 };
                xc * yc * zc
            }
            SelectionMode::Range => {
                let x_lo = state.x_min.saturating_sub(1) as usize;
                let x_hi = state.x_max.min(DIM_X as u32).saturating_sub(1) as usize;
                let y_lo = state.y_min.saturating_sub(1) as usize;
                let y_hi = state.y_max.min(DIM_Y as u32).saturating_sub(1) as usize;
                let z_lo = state.z_min.saturating_sub(1) as usize;
                let z_hi = state.z_max.min(DIM_Z as u32).saturating_sub(1) as usize;
                (x_hi - x_lo + 1) * (y_hi - y_lo + 1) * (z_hi - z_lo + 1)
            }
        };
        **text = format!("Showing {} cubes", count);
    }
}

/// Syncs clamped slider values back to SliderValue components so UI stays consistent.
pub fn sync_range_sliders(
    state: Res<RangeSelectionState>,
    sliders: Query<(Entity, &SliderValue, &RangeSliderAxis), With<CubeGridSlider>>,
    mut commands: Commands,
) {
    if !state.is_changed() {
        return;
    }
    for (entity, value, axis) in &sliders {
        let new_val = match axis {
            RangeSliderAxis::XMin => state.x_min as f32,
            RangeSliderAxis::XMax => state.x_max as f32,
            RangeSliderAxis::YMin => state.y_min as f32,
            RangeSliderAxis::YMax => state.y_max as f32,
            RangeSliderAxis::ZMin => state.z_min as f32,
            RangeSliderAxis::ZMax => state.z_max as f32,
        };
        if (value.0 - new_val).abs() > 0.5 {
            commands.entity(entity).insert(SliderValue(new_val));
        }
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
