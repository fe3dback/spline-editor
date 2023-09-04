use bevy::{prelude::*, text::Text, time::Time};

use crate::consts::*;
use crate::resources::*;
use crate::StatusBarTextMarker;

pub fn draw_plot(mut gz: Gizmos, area: Res<Area>) {
    let offset2d = Vec2::splat(OFFSET);
    let point_size = Vec2::splat(5.0);

    // draw outline
    let tl = screen(offset2d);
    let br = screen(
        Vec2 {
            x: PLOT_WIDTH,
            y: PLOT_HEIGHT,
        } - offset2d,
    );
    draw_box(&mut gz, tl, br, Color::GRAY);

    // draw axis Y
    for axis_y in vec![0.25, 0.5, 0.75] {
        let left = plot(Vec2::new(0.0, axis_y));
        let rigth = plot(Vec2::new(1.0, axis_y));

        gz.line_2d(left, rigth, color_light(0.05));
    }

    // draw axis X
    for axis_x in vec![0.2, 0.4, 0.5, 0.6, 0.8] {
        let top: Vec2 = plot(Vec2::new(axis_x, 0.0));
        let bottom = plot(Vec2::new(axis_x, 1.0));

        gz.line_2d(top, bottom, color_light(0.03));
    }

    // draw points
    let mut prev_norm = Vec2::ZERO;
    let mut prev = Vec2::ZERO;

    let mut points = area.points.clone();
    points.sort_by(|a, b| a.uncommited.x.partial_cmp(&b.uncommited.x).unwrap());

    let mut index = -1;
    for p in points.iter() {
        let norm = match p.selected {
            false => p.commited,
            true => p.uncommited,
        };

        let cur = plot(norm);
        let mut color = Color::GOLD;

        if p.selected {
            color = Color::WHITE;
        }

        draw_box(&mut gz, cur - point_size, cur + point_size, color);

        index += 1;
        if index == 0 {
            prev_norm = norm;
            prev = cur;
            continue;
        }

        gz.line_gradient_2d(prev, cur, color_axis_x(prev_norm.x), color_axis_x(norm.x));
        prev = cur;
        prev_norm = norm;
    }
}

pub fn draw_status_bar(
    mut text_query: Query<&mut Text, With<StatusBarTextMarker>>,
    mut gz: Gizmos,
    mut status_bar: ResMut<StatusBar>,
    time: Res<Time>,
    mouse_plot: Res<MousePlot>,
    file_info: Res<AttachedFile>,
    area: Res<Area>,
) {
    let tl = screen(Vec2::new(0.0, PLOT_HEIGHT));
    let tr = screen(Vec2::new(PLOT_WIDTH, PLOT_HEIGHT));

    // draw background
    gz.line_2d(tl, tr, Color::DARK_GRAY);

    // priority 1: basic info
    let mut text =
        String::from("drag-and-drop file (to attach), or copy-paste raw file content to edit..");
    let mut color = Color::DARK_GRAY;

    // priority 2: file info
    if file_info.attached {
        text = format!(
            "file: {}{}",
            file_info.file_path,
            match file_info.dirty {
                true => "* (changed)",
                false => "",
            }
        )
    }

    // priority 3: override with important text
    let important = status_bar.most_important_text_display(time.delta_seconds());
    if !important.0.is_empty() {
        text = important.0;
        color = important.1;
    }

    // add coords
    let mut moving_point: Option<Point> = None;
    for p in area.points.iter() {
        if p.selected {
            moving_point = Some(*p);
            break;
        }
    }

    let coords_text = match moving_point {
        Some(p) => format!(
            "moving=[{:.2},{:.2}]->[{:.2},{:.2}]",
            p.commited.x, p.commited.y, p.uncommited.x, p.uncommited.y
        ),
        None => format!("[{:.1},{:.1}]", mouse_plot.coords.x, mouse_plot.coords.y),
    };

    text = format!("{} {}", coords_text, text);

    // change text
    for mut status_bar_text in &mut text_query {
        status_bar_text.sections[0].value = text;
        status_bar_text.sections[0].style.color = color;
        return;
    }
}

pub fn draw_ui(mut gz: Gizmos, mouse_plot: Res<MousePlot>, area: Res<Area>) {
    if area.has_moving_points() {
        return;
    }

    // highlight selectable points
    let closest = area.closest(mouse_plot.coords);
    let dist = Vec2::new(closest.commited.x, closest.commited.y).distance(mouse_plot.coords);
    if dist <= ACTIVE_RADIUS {
        gz.circle_2d(plot(closest.commited), 10.0, Color::YELLOW_GREEN);
    }

    // interpolated pos of possible new point
    let new_point_ghost = area.interpolate(mouse_plot.coords.x);
    if mouse_plot.coords.distance(new_point_ghost) <= (ACTIVE_RADIUS * 1.1) {
        draw_box(
            &mut gz,
            plot(new_point_ghost) - Vec2::splat(5.0),
            plot(new_point_ghost) + Vec2::splat(5.0),
            Color::SEA_GREEN,
        );
    }
}

fn draw_box(gz: &mut Gizmos, tl: Vec2, br: Vec2, c: Color) {
    gz.linestrip_2d(
        vec![tl, Vec2::new(br.x, tl.y), br, Vec2::new(tl.x, br.y), tl],
        c,
    )
}

fn color_axis_x(v: f32) -> Color {
    Color::hsl(v * 360.0, 0.75, 0.5)
}

fn color_light(lightness: f32) -> Color {
    Color::hsl(0.0, 0.0, lightness)
}
