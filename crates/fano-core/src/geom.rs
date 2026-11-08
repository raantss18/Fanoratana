//! Géométrie minimale, sans dépendance, en points de page (1 pt = 1/72 pouce).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub type Vec2 = Point;

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn dist(self, o: Point) -> f32 {
        (self - o).len()
    }
    pub fn len(self) -> f32 {
        self.x.hypot(self.y)
    }
    pub fn norm(self) -> Point {
        let l = self.len();
        if l < 1e-6 {
            Point::new(0.0, 0.0)
        } else {
            Point::new(self.x / l, self.y / l)
        }
    }
    pub fn perp(self) -> Point {
        Point::new(-self.y, self.x)
    }
    pub fn dot(self, o: Point) -> f32 {
        self.x * o.x + self.y * o.y
    }
    pub fn cross(self, o: Point) -> f32 {
        self.x * o.y - self.y * o.x
    }
    pub fn lerp(self, o: Point, t: f32) -> Point {
        Point::new(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t)
    }
}

impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, o: Point) -> Point {
        Point::new(self.x + o.x, self.y + o.y)
    }
}
impl std::ops::Sub for Point {
    type Output = Point;
    fn sub(self, o: Point) -> Point {
        Point::new(self.x - o.x, self.y - o.y)
    }
}
impl std::ops::Mul<f32> for Point {
    type Output = Point;
    fn mul(self, k: f32) -> Point {
        Point::new(self.x * k, self.y * k)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub const NOTHING: Rect = Rect {
        min: Point::new(f32::INFINITY, f32::INFINITY),
        max: Point::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
    };

    pub fn from_points(pts: impl IntoIterator<Item = Point>) -> Rect {
        let mut r = Rect::NOTHING;
        for p in pts {
            r.add(p);
        }
        r
    }
    pub fn from_min_size(min: Point, w: f32, h: f32) -> Rect {
        Rect {
            min,
            max: Point::new(min.x + w, min.y + h),
        }
    }
    pub fn add(&mut self, p: Point) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }
    pub fn union(self, o: Rect) -> Rect {
        Rect {
            min: Point::new(self.min.x.min(o.min.x), self.min.y.min(o.min.y)),
            max: Point::new(self.max.x.max(o.max.x), self.max.y.max(o.max.y)),
        }
    }
    pub fn is_valid(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y
    }
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
    pub fn center(&self) -> Point {
        self.min.lerp(self.max, 0.5)
    }
    pub fn expand(self, m: f32) -> Rect {
        Rect {
            min: Point::new(self.min.x - m, self.min.y - m),
            max: Point::new(self.max.x + m, self.max.y + m),
        }
    }
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }
    pub fn intersects(&self, o: &Rect) -> bool {
        self.min.x <= o.max.x && o.min.x <= self.max.x && self.min.y <= o.max.y && o.min.y <= self.max.y
    }
    pub fn translate(self, d: Point) -> Rect {
        Rect {
            min: self.min + d,
            max: self.max + d,
        }
    }
}
