use kurbo::{Point, Rect, Vec2};
use nalgebra::{Rotation3, Vector3};

pub struct Quad {
  pub p: [Point; 4],
}

impl From<Rect> for Quad {
  fn from(rect: Rect) -> Self {
    Quad {
      p: [
        Point::new(rect.x1, rect.y0),
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x0, rect.y1),
        Point::new(rect.x1, rect.y1),
      ],
    }
  }
}

impl Quad {
  pub fn new_tilted(
    rect: kurbo::Rect,
    cursor_pos: Point,
    max_tilt_rad: f64,
    camera_dist: f64,
  ) -> Quad {
    let center = rect.center() - Vec2::new(rect.min_x(), rect.min_y());

    // Normalize cursor over button to [-1, 1]
    let nx = ((cursor_pos.x - rect.x0) / rect.width()) * 2.0 - 1.0;
    let ny = ((cursor_pos.y - rect.y0) / rect.height()) * 2.0 - 1.0;

    // Map to small rotations. Signs here are “feel” choices.
    let rot_y = nx * max_tilt_rad; // left/right tilt
    let rot_x = ny * max_tilt_rad; // up/down tilt

    let rot = Rotation3::from_euler_angles(rot_x, rot_y, 0.0);

    // Corners around center, in local 2D (y down). Convert to 3D with y up.
    let corners = [
      Vector3::new(-center.x, center.y, 0.0),  // top-left
      Vector3::new(center.x, center.y, 0.0),   // top-right
      Vector3::new(center.x, -center.y, 0.0),  // bottom-right
      Vector3::new(-center.x, -center.y, 0.0), // bottom-left
    ];

    let mut out = [Point::new(0.0, 0.0); 4];
    for (i, v) in corners.into_iter().enumerate() {
      // Rotate in 3D
      let r = rot * v;

      // Simple perspective: x' = x * d/(d - z), y' = y * d/(d - z)
      // (z toward camera makes it bigger)
      let denom = camera_dist - r.z;
      let s = camera_dist / denom;

      // Back to screen coords (y down) and translate to button center on screen
      let screen_cx = rect.min_x() + center.x;
      let screen_cy = rect.min_y() + center.y;

      out[i] = Point::new(screen_cx + r.x * s, screen_cy - r.y * s);
    }

    Quad { p: out }
  }
}

impl kurbo::Shape for Quad {
  type PathElementsIter<'iter> = std::array::IntoIter<kurbo::PathEl, 5>;

  fn bounding_box(&self) -> kurbo::Rect {
    kurbo::Rect::new(
      self.p.iter().map(|p| p.x).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
      self.p.iter().map(|p| p.y).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
      self.p.iter().map(|p| p.x).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
      self.p.iter().map(|p| p.y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
    )
  }
  fn to_path(&self, tolerance: f64) -> kurbo::BezPath {
    kurbo::BezPath::from_iter(self.path_elements(tolerance))
  }

  fn path_elements(&self, _tolerance: f64) -> Self::PathElementsIter<'_> {
    [
      kurbo::PathEl::MoveTo(self.p[0]),
      kurbo::PathEl::LineTo(self.p[1]),
      kurbo::PathEl::LineTo(self.p[2]),
      kurbo::PathEl::LineTo(self.p[3]),
      kurbo::PathEl::ClosePath,
    ]
    .into_iter()
  }

  // TODO: These are all wrong.
  fn winding(&self, _pt: Point) -> i32 { 0 }
  fn perimeter(&self, _accuracy: f64) -> f64 { 0.0 }
  fn area(&self) -> f64 { 0.0 }
}
