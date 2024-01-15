use sfml::graphics::{Color, RenderTarget, RenderWindow, Shape};
use sfml::graphics::ConvexShape;
use sfml::system::Vector2f;
use sfml::window::Style;

use log::debug;

use crate::config;

pub fn create_window() -> RenderWindow {
    debug!("Creating window");
    let resolution = config::get_resolution(); 
    let window = RenderWindow::new(resolution, "SFML window", Style::NONE, &Default::default());
    window

}

/// this function will render our triangles to the screen
pub fn draw_triangles(window: &mut RenderWindow, triangles: &Vec<geo::Triangle<f64>>, scale: f32, resolution: (u32, u32), color: Option<Color>) {
    // debug!("Drawing triangles! len: {}", triangles.len());
    let c = match color {
        Some(c) => c,
        None => {
            debug!("No color provided, defaulting to Yellow");
            Color::YELLOW
        },
    };
    for tri in triangles {
        // debug!("Drawing triangle: ({}, {}), ({}, {}), ({}, {})", tri.0.x, tri.0.y, tri.1.x, tri.1.y, tri.2.x, tri.2.y);
        let mut triangle = ConvexShape::new(3);
        let p0_0 = tri.0.x as f32;
        let p0_1 = tri.0.y as f32;
        let p1_0 = tri.1.x as f32;
        let p1_1 = tri.1.y as f32;
        let p2_0 = tri.2.x as f32;
        let p2_1 = tri.2.y as f32;

        let mid_point = (resolution.0 as f32 / 2.0, resolution.1 as f32 / 2.0);
        let p0 = (p0_0 - mid_point.0, p0_1 - mid_point.1);
        let p1 = (p1_0 - mid_point.0, p1_1 - mid_point.1);
        let p2 = (p2_0 - mid_point.0, p2_1 - mid_point.1);

        let p0 = (p0.0 * scale, p0.1 * scale);
        let p1 = (p1.0 * scale, p1.1 * scale);
        let p2 = (p2.0 * scale, p2.1 * scale);

        let p0 = (p0.0 + mid_point.0, p0.1 + mid_point.1);
        let p1 = (p1.0 + mid_point.0, p1.1 + mid_point.1);
        let p2 = (p2.0 + mid_point.0, p2.1 + mid_point.1);

        triangle.set_point(0, Vector2f::new(p0.0, p0.1));
        triangle.set_point(1, Vector2f::new(p1.0, p1.1));
        triangle.set_point(2, Vector2f::new(p2.0, p2.1));



 //       triangle.set_point(0, Vector2f::new(tri.0.x as f32, tri.0.y as f32));
 //       triangle.set_point(1, Vector2f::new(tri.1.x as f32, tri.1.y as f32));
 //       triangle.set_point(2, Vector2f::new(tri.2.x as f32, tri.2.y as f32));
        triangle.set_fill_color(c);

        window.draw(&triangle);
    }

}