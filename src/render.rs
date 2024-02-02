use sfml::graphics::{Color, Drawable, Font, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Shape, Transformable, Vertex, VertexBuffer};
use sfml::graphics::ConvexShape;
use sfml::system::Vector2f;
use sfml::window::Style;

use log::{debug, info};
use sfml::SfBox;

use crate::config;
use crate::geometry::{Plotable, DepthLayer};


pub fn create_window() -> RenderWindow {
    debug!("Creating window");
    let resolution = config::get_resolution(); 
    let window = RenderWindow::new(resolution, "SFML window", Style::NONE, &Default::default());
    window

}

pub fn render_objects<T: Plotable>(window: &mut RenderWindow, plotvec: &Vec<T>, zoom: f32, resolution: (u32, u32), viewvec: (f32, f32)) {

        // render code
        for plot in plotvec {
            plot.render(window, zoom, viewvec, resolution)
        }
}

/// this function will render our triangles to the screen
pub fn draw_triangles(window: &mut RenderWindow, triangles: &Vec<geo::Triangle<f64>>, zoom: f32, resolution: (u32, u32), viewvec: (f32, f32), color: Option<Color>) {
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

        let p0 = (p0.0 * zoom, p0.1 * zoom);
        let p1 = (p1.0 * zoom, p1.1 * zoom);
        let p2 = (p2.0 * zoom, p2.1 * zoom);

        let p0 = (p0.0 + mid_point.0 + viewvec.0, p0.1 + mid_point.1 + viewvec.1);
        let p1 = (p1.0 + mid_point.0 + viewvec.0, p1.1 + mid_point.1 + viewvec.1);
        let p2 = (p2.0 + mid_point.0 + viewvec.0, p2.1 + mid_point.1 + viewvec.1);

        triangle.set_point(0, Vector2f::new(p0.0, p0.1));
        triangle.set_point(1, Vector2f::new(p1.0, p1.1));
        triangle.set_point(2, Vector2f::new(p2.0, p2.1));



        triangle.set_fill_color(c);

        window.draw(&triangle);
    }

}

pub fn draw_vertex_buffer(window: &mut RenderWindow, vertex_buffer: &VertexBuffer) {
    info!("Drawing {} Vertices!", vertex_buffer.vertex_count());
    window.draw(vertex_buffer);
}

pub fn draw_vertex_vector(window: &mut RenderWindow, vertices: &Vec<Vertex>) {
    window.draw_primitives(vertices, PrimitiveType::TRIANGLES, &RenderStates::default());

}


pub fn render_soundg(window: &mut RenderWindow, depth_soundings: &DepthLayer, resolution: (u32, u32), zoom: f32, font: &SfBox<Font>) {
    for sounding in &depth_soundings.coordinates {
        let mut text = sfml::graphics::Text::new(&format!("{:.1}", sounding.2 * 3.281), &font, 10);
        
        //let pos = (sounding.0 as f32, sounding.1 as f32);
        //let mid_point = (resolution.0 as f32 / 2.0, resolution.1 as f32 / 2.0);
        //let pos = (pos.0 - mid_point.0, pos.1 - mid_point.1);
        //let pos = (pos.0 * zoom, pos.1 * zoom);
        //let pos = (pos.0 + mid_point.0, pos.1 + mid_point.1);
        let pos = dbg!((sounding.0 as f32, sounding.1 as f32));
        
        text.set_position(pos);
        text.set_fill_color(Color::WHITE);
        window.draw(&text);
    }
}