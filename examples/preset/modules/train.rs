use correct_bar::{
  bar::{Color, ModuleImpl, Updater},
  math::{Pos, Rect},
};
use parking_lot::Mutex;
use std::time::Duration;

pub struct Train {
  pub primary: Color,
  pub pos:     Mutex<i32>,
}

impl Clone for Train {
  fn clone(&self) -> Self { Train { primary: self.primary, pos: Mutex::new(*self.pos.lock()) } }
}

impl ModuleImpl for Train {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let pos = {
      let mut pos = self.pos.lock();
      if *pos <= -65 {
        *pos = 100;
      }
      *pos -= 1;
      *pos
    };

    macro_rules! pos {
      ( $x:expr, $y:expr ) => {
        Pos { x: $x + pos as i32, y: $y }
      };
    }

    ctx.advance_by(100);
    ctx.set_max_width(100);

    // main car
    ctx.draw_triangle(pos!(20, 10), pos!(10, 23), pos!(20, 23), self.primary);
    ctx.draw_rect(Rect { pos: pos!(20, 10), width: 20, height: 12 }, self.primary);

    // smoke stack
    ctx.draw_rect(Rect { pos: pos!(18, 7), width: 5, height: 3 }, self.primary);

    let smoke_1 = if (pos % 2).abs() > 0 { 1 } else { 0 };
    let smoke_2 = if (pos % 2).abs() > 0 { 0 } else { 1 };
    // smoke
    ctx.draw_rect(Rect { pos: pos!(20, 5), width: 3, height: 1 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(21, 4), width: 3, height: 1 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(23, 3), width: 3, height: 1 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(26, 2 + smoke_1), width: 4, height: 1 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(30, 2 + smoke_2), width: 4, height: 1 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(34, 2 + smoke_1), width: 4, height: 1 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(38, 2 + smoke_2), width: 4, height: 1 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(42, 2 + smoke_1), width: 4, height: 1 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(46, 2 + smoke_2), width: 4, height: 1 }, self.primary);

    // thing on the back of the main car
    ctx.draw_rect(Rect { pos: pos!(30, 8), width: 12, height: 5 }, self.primary);

    // wheels
    ctx.draw_rect(Rect { pos: pos!(23, 22), width: 3, height: 3 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(23 + 6, 22), width: 3, height: 3 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(23 + 12, 22), width: 3, height: 3 }, self.primary);

    // back car
    ctx.draw_rect(Rect { pos: pos!(45, 12), width: 20, height: 10 }, self.primary);
    // car conector
    ctx.draw_rect(Rect { pos: pos!(35, 18), width: 10, height: 3 }, self.primary);
    // back car wheels
    ctx.draw_rect(Rect { pos: pos!(45 + 3, 22), width: 3, height: 3 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(45 + 9, 22), width: 3, height: 3 }, self.primary);
    ctx.draw_rect(Rect { pos: pos!(45 + 15, 22), width: 3, height: 3 }, self.primary);

    //});
  }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}
