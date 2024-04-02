use sfml::graphics::{Color, Font, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Transformable, Vertex};
use sfml::window::Style;

use log::debug;
use sfml::SfBox;

use crate::config;
use crate::geometry::{Plotable, DepthLayer};

pub fn create_window() -> RenderWindow {
    debug!("Creating window");
    let resolution = config::get_resolution(); 
    let window = RenderWindow::new(resolution, "SFML window", Style::NONE, &Default::default());
    window

}

pub fn render_objects<T: Plotable>(window: &mut RenderWindow, plotvec: &Vec<T>) {

        // render code
        for plot in plotvec {
            plot.render(window)
        }
}

//pub fn draw_vertex_buffer(window: &mut RenderWindow, vertex_buffer: &VertexBuffer) {
//    info!("Drawing {} Vertices!", vertex_buffer.vertex_count());
//    window.draw(vertex_buffer);
//}

pub fn draw_vertex_vector(window: &mut RenderWindow, vertices: &Vec<Vertex>) {
    window.draw_primitives(vertices, PrimitiveType::TRIANGLES, &RenderStates::default());

}

pub fn render_soundg(window: &mut RenderWindow, depth_soundings: &DepthLayer,  font: &SfBox<Font>, scale: f32) {
    for sounding in &depth_soundings.coordinates {
        let mut text = sfml::graphics::Text::new(&format!("{:.1}", sounding.2 * 3.281), &font, 8);
        
        let pos = (sounding.0 as f32, sounding.1 as f32);
        
        text.set_position(pos);
        text.set_fill_color(Color::WHITE);
        text.scale((2.0 * scale, 2.0 * scale));
        window.draw(&text);
    }
}
