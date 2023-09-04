use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct AttachedFile {
    pub attached: bool,
    pub file_path: String,
    pub dirty: bool,
    pub state: Vec<Point>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub commited: Vec2,
    pub uncommited: Vec2,
    pub selected: bool,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            commited: Vec2::new(x, y),
            uncommited: Vec2::new(x, y),
            selected: false,
        }
    }
}

#[derive(Resource)]
pub struct Area {
    pub points: Vec<Point>,
}

#[derive(Resource, Default)]
pub struct StatusBar {
    pub error: StatusTimedText,
    pub info: StatusTimedText,
    pub hint: String,
}

#[derive(Default)]
pub struct StatusTimedText {
    pub text: String,
    pub ttl: f32,
}

#[derive(Resource, Default)]
pub struct MousePlot {
    pub coords: Vec2,
}

impl Area {
    pub fn new() -> Self {
        Self {
            points: vec![
                Point::new(0.0, 0.5),
                Point::new(0.2, 0.3),
                Point::new(0.4, 0.5),
                Point::new(0.8, 0.9),
                Point::new(0.85, 0.05),
                Point::new(1.0, 0.5),
            ],
        }
    }

    pub fn insert(&mut self, p: Vec2, select_created: bool) {
        if p.x <= 0.0 {
            return;
        }
        if p.x >= 1.0 {
            return;
        }

        self.points.push(Point {
            commited: p,
            uncommited: p,
            selected: select_created,
        })
    }

    pub fn delete(&mut self, p: Vec2) {
        if p.x <= 0.0 {
            return;
        }
        if p.x >= 1.0 {
            return;
        }

        let exist_points = self.points.clone();
        self.points.clear();

        for exist_point in exist_points {
            if exist_point.commited == p {
                continue;
            }

            self.points.push(exist_point);
        }
    }

    pub fn closest(&self, coord: Vec2) -> Point {
        let mut points = self.points.clone();

        points.sort_by(|a, b| {
            let dist1 = coord.distance(a.commited);
            let dist2 = coord.distance(b.commited);

            dist1.partial_cmp(&dist2).unwrap()
        });

        *points.first().unwrap()
    }

    pub fn has_moving_points(&self) -> bool {
        for p in self.points.clone() {
            if p.selected {
                return true;
            }
        }

        return false;
    }

    pub fn interpolate(&self, x: f32) -> Vec2 {
        let mut points = self.points.clone();
        points.sort_by(|a, b| a.commited.x.partial_cmp(&b.commited.x).unwrap());

        if x <= 0.0 {
            return points.first().unwrap().commited;
        }
        if x >= 1.0 {
            return points.last().unwrap().commited;
        }

        let mut left = *points.first().unwrap();
        let mut right = *points.last().unwrap();
        for p in points {
            if p.commited.x >= left.commited.x && p.commited.x <= x {
                left = p;
            }
            if p.commited.x < right.commited.x && p.commited.x > x {
                right = p;
            }
        }

        let delta = (x - left.commited.x) / (right.commited.x - left.commited.x);
        left.commited.lerp(right.commited, delta)
    }
}

impl StatusBar {
    pub fn show_error(&mut self, err: &str) {
        self.error = StatusTimedText {
            text: String::from(err),
            ttl: 5.0,
        }
    }

    pub fn show_info(&mut self, info: &str) {
        self.info = StatusTimedText {
            text: String::from(info),
            ttl: 3.0,
        }
    }

    pub fn show_hint(&mut self, hint: String) {
        self.hint = hint;
    }

    pub fn most_important_text_display(&mut self, delta_time: f32) -> (String, Color) {
        if self.error.ttl > 0.0 {
            let newttl = (self.error.ttl - delta_time).clamp(0.0, 10.0);
            self.error.ttl = newttl;

            return (self.format_timed(&self.error), Color::ORANGE_RED);
        }

        if self.info.ttl > 0.0 {
            let newttl = (self.info.ttl - delta_time).clamp(0.0, 5.0);
            self.info.ttl = newttl;
            return (self.format_timed(&self.info), Color::AQUAMARINE);
        }

        let hint = self.hint.clone();
        self.hint.clear();

        return (hint, Color::GRAY);
    }

    fn format_timed(&self, text: &StatusTimedText) -> String {
        format!("{} ({:.1}s)", text.text, text.ttl).to_string()
    }
}
