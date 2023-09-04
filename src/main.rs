mod consts;
mod draw;
mod encoders;
mod resources;

use std::path::PathBuf;

use arboard::Clipboard;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    text::{Text, Text2dBundle},
    utils::default,
    window::{FileDragAndDrop, PresentMode, Window, WindowMode, WindowPlugin},
    DefaultPlugins,
};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use consts::*;
use draw::*;
use encoders::{decode, encode};
use resources::*;

// Some fasty-shitty code here, but its works fine..
// In case of refactoring wish:
// - remove plotWH and left only windowWH (not need to scale all to *2)
// - separate modules with events (instead of spaghetti code)
// - add CQRS (user interactions should push migration actions (with apply/revert code), instead of direct data change)
// - - add undo/redo

#[derive(Component)]
pub struct StatusBarTextMarker;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Curve editor (ctrl+c/v for copy/paste data)"),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        present_mode: PresentMode::AutoVsync,
                        mode: WindowMode::Windowed,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
        )
        .insert_resource(Area::new())
        .insert_resource(AttachedFile::default())
        .insert_resource(StatusBar::default())
        .insert_resource(MousePlot::default())
        .add_systems(Startup, init)
        .add_systems(Update, (file_attach, clipboard))
        .add_systems(
            Update,
            (
                update_mouse_plot_coords,
                select_points,
                delete_points,
                create_points,
                move_points,
                set_dirty_state,
                save_file,
            ),
        )
        .add_systems(Update, (draw_plot, draw_ui, draw_status_bar))
        .run();
}

fn init(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/font.ttf");

    cmd.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        ..default()
    });

    cmd.spawn((
        StatusBarTextMarker,
        Text2dBundle {
            text: Text::from_section(
                "...",
                bevy::text::TextStyle {
                    color: Color::DARK_GRAY,
                    font: font,
                    font_size: 18.0,
                    ..default()
                },
            ),
            text_anchor: bevy::sprite::Anchor::CenterLeft,
            transform: Transform::from_translation(
                Vec2::new(
                    -(WINDOW_WIDTH * 0.5) + (OFFSET * 2.0),
                    -(WINDOW_HEIGHT * 0.5) + STATUS_BAR_HEIGHT - (STATUS_BAR_HEIGHT * 0.5),
                )
                .extend(0.0),
            ),
            ..default()
        },
    ));
}

fn clipboard(
    mut area: ResMut<Area>,
    mut status_bar: ResMut<StatusBar>,
    keyboard: Res<Input<KeyCode>>,
) {
    if !keyboard.pressed(KeyCode::ControlLeft) {
        return;
    }

    #[derive(PartialEq, Eq)]
    enum Action {
        Nothing,
        Copy,
        Paste,
    }

    let mut act = Action::Nothing;
    if keyboard.just_pressed(KeyCode::C) {
        act = Action::Copy;
    }
    if keyboard.just_pressed(KeyCode::V) {
        act = Action::Paste;
    }

    if act == Action::Nothing {
        return;
    }

    let mut ctx = match Clipboard::new() {
        Ok(cb) => cb,
        Err(err) => {
            status_bar.show_error(format!("clipboard not available: {}", err).as_str());
            return;
        }
    };

    match act {
        Action::Copy => {
            let mut data: Vec<Vec2> = vec![];
            for p in area.points.clone() {
                data.push(p.commited);
            }
            let content = encode(data);

            match ctx.set_text(content) {
                Err(err) => {
                    status_bar.show_error(format!("copy failed: {}", err).as_str());
                }
                _ => {}
            };
        }
        Action::Paste => match ctx.get_text() {
            Ok(content) => {
                apply_content(&mut area, &mut status_bar, &content);
            }
            Err(err) => {
                status_bar.show_error(format!("can`t paste content: {}", err).as_str());
            }
        },
        _ => {}
    }
}

fn file_attach(
    mut area: ResMut<Area>,
    mut events: EventReader<FileDragAndDrop>,
    mut file: ResMut<AttachedFile>,
    mut status_bar: ResMut<StatusBar>,
) {
    for ev in events.iter() {
        match ev {
            FileDragAndDrop::DroppedFile {
                window: _,
                path_buf,
            } => {
                let data = match std::fs::read(path_buf) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        status_bar
                            .show_error(format!("can`t open: {} ({:?})", err, path_buf).as_str());
                        return;
                    }
                };

                let data = match std::str::from_utf8(&data) {
                    Ok(s) => s,
                    Err(err) => {
                        status_bar.show_error(
                            format!("unexpected content: {} ({:?})", err, path_buf).as_str(),
                        );
                        return;
                    }
                };

                let changed = apply_content(&mut area, &mut status_bar, data);
                if !changed {
                    return;
                }

                file.state = area.points.clone();
                file.attached = true;
                file.dirty = false;
                file.file_path = path_buf.to_str().unwrap().to_string();
            }
            _ => {}
        }
    }
}

fn apply_content(area: &mut ResMut<Area>, status_bar: &mut ResMut<StatusBar>, data: &str) -> bool {
    let content = match decode(String::from(data)) {
        Ok(points) => points,
        Err(err) => {
            status_bar.show_error(format!("invalid format: {}", err).as_str());
            return false;
        }
    };

    let mut points: Vec<Point> = vec![];
    for p in content {
        points.push(Point::new(p.x, p.y));
    }

    area.points = points;
    return true;
}

fn update_mouse_plot_coords(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_res: ResMut<MousePlot>,
) {
    for ev in cursor_moved_events.iter() {
        match ev {
            CursorMoved {
                window: _,
                position,
            } => {
                let abs_pos_on_plot = *position - Vec2::splat(OFFSET * WINDOW_SCALE);
                let rel_pos = abs_pos_on_plot
                    / Vec2::new(
                        (PLOT_WIDTH * WINDOW_SCALE) - (OFFSET * 2.0 * WINDOW_SCALE),
                        (PLOT_HEIGHT * WINDOW_SCALE) - (OFFSET * 2.0 * WINDOW_SCALE),
                    );

                let rel_pos = rel_pos.clamp(Vec2::ZERO, Vec2::ONE);
                mouse_res.coords = Vec2::new(roundf32(rel_pos.x, 2), 1.0 - roundf32(rel_pos.y, 2));
            }
        }
    }
}

fn select_points(
    mut area: ResMut<Area>,
    mouse_res: Res<MousePlot>,
    mouse_input: Res<Input<MouseButton>>,
) {
    // clear selection
    if mouse_input.just_released(MouseButton::Left) {
        for p in area.points.iter_mut() {
            if !p.selected {
                continue;
            }

            p.selected = false;
            p.commited = p.uncommited;
        }
        return;
    }

    // try select
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    // find closest in distance
    let closest = area.closest(mouse_res.coords);
    let dist = Vec2::new(closest.commited.x, closest.commited.y).distance(mouse_res.coords);
    if dist > ACTIVE_RADIUS {
        return;
    }

    // set active
    for p in area.points.iter_mut() {
        if p.clone() == closest {
            p.selected = true;
        }
    }
}

fn move_points(mut area: ResMut<Area>, mouse_res: Res<MousePlot>, keyboard: Res<Input<KeyCode>>) {
    #[derive(PartialEq)]
    enum Axis {
        Both,
        OnlyX,
        OnlyY,
    }

    let mut axis = Axis::Both;

    if keyboard.pressed(KeyCode::X) {
        axis = Axis::OnlyX
    } else if keyboard.pressed(KeyCode::Y) {
        axis = Axis::OnlyY
    }

    let snap = keyboard.pressed(KeyCode::ControlLeft);

    for p in area.points.iter_mut() {
        if !p.selected {
            continue;
        }

        // reset
        p.uncommited = p.commited;

        // move
        if axis != Axis::OnlyY {
            p.uncommited.x = mouse_res.coords.x;
        }
        if axis != Axis::OnlyX {
            p.uncommited.y = mouse_res.coords.y;
        }

        // snap
        if snap {
            p.uncommited.x = roundf32(p.uncommited.x, 1);
            p.uncommited.y = roundf32(p.uncommited.y, 1);
        }

        // clamp
        p.uncommited.x = p.uncommited.x.clamp(0.01, 0.99);

        // reset start/end
        if p.commited.x == 0.0 {
            p.uncommited.x = 0.0;
        }
        if p.commited.x == 1.0 {
            p.uncommited.x = 1.0;
        }
    }
}

fn create_points(
    mut area: ResMut<Area>,
    mouse_res: Res<MousePlot>,
    mouse_input: Res<Input<MouseButton>>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    if area.has_moving_points() {
        return;
    }

    // find future point place
    let ghost = area.interpolate(mouse_res.coords.x);
    let closest = area.closest(mouse_res.coords);

    // new point very close to exist
    if closest.commited.distance(ghost) < (ACTIVE_RADIUS * 1.1) {
        return;
    }

    // mouse far away of new potential point (just click around plot)
    if mouse_res.coords.distance(ghost) > ACTIVE_RADIUS {
        return;
    }

    // add to area
    area.insert(ghost, true);
}

fn delete_points(
    mut area: ResMut<Area>,
    mouse_res: Res<MousePlot>,
    mouse_input: Res<Input<MouseButton>>,
) {
    if !mouse_input.just_pressed(MouseButton::Right) {
        return;
    }

    if area.has_moving_points() {
        return;
    }

    // find point under cursor
    let closest = area.closest(mouse_res.coords);
    if closest.commited.distance(closest.commited) >= ACTIVE_RADIUS {
        return;
    }

    // mouse far away of deleted point (just click around plot)
    if mouse_res.coords.distance(closest.commited) > ACTIVE_RADIUS {
        return;
    }

    area.delete(closest.commited);
}

fn set_dirty_state(mut file: ResMut<AttachedFile>, area: Res<Area>) {
    // check if already dirty
    if file.dirty {
        return;
    }

    if is_dirty(&file.state, &area.points) {
        file.dirty = true;
        return;
    }
}

fn is_dirty(prev: &Vec<Point>, next: &Vec<Point>) -> bool {
    if prev.len() != next.len() {
        return true;
    }

    for n in 0..prev.len() {
        let p1 = prev[n];
        let p2 = next[n];

        if p1.commited.x != p2.commited.x {
            return true;
        }
        if p1.commited.y != p2.commited.y {
            return true;
        }
    }

    return false;
}

fn save_file(
    mut file: ResMut<AttachedFile>,
    mut status_bar: ResMut<StatusBar>,
    area: Res<Area>,
    keyboard: Res<Input<KeyCode>>,
) {
    if !keyboard.pressed(KeyCode::ControlLeft) {
        return;
    }

    if !keyboard.just_pressed(KeyCode::S) {
        return;
    }

    if !file.dirty {
        return;
    }

    if !file.attached {
        return;
    }

    let mut data: Vec<Vec2> = vec![];
    for p in area.points.clone() {
        data.push(p.commited);
    }

    let content = encode(data);
    match std::fs::write(PathBuf::from(file.file_path.clone()), content) {
        Err(err) => status_bar.show_error(format!("failed save: {}", err).as_str()),
        _ => status_bar.show_info(format!("file {} saved!", file.file_path).as_str()),
    }

    file.dirty = false;
    file.state = area.points.clone();
}
