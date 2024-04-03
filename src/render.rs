use sfml::graphics::{Color, Font, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Transformable, Vertex, View};
use sfml::window::Style;
use log::debug;
use sfml::SfBox;

use crate::config;
use crate::geometry::{DepthLayer, LayerExtent, Plotable, DEPARE};

use std::collections::HashMap;

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

pub fn get_zoom(map: &HashMap<u16, Vec<DEPARE>>, view: &SfBox<View>) -> ((f32, f32), f32) {
    let mut extents = Vec::new();
    for (_, depares) in map {
        for depare in depares {
            extents.push(depare.extent.clone());
        }   
    }    
    let min_x = extents.iter().fold(f32::INFINITY, |acc, x| acc.min(x.MinX));
    let max_x = extents.iter().fold(f32::NEG_INFINITY, |acc, x| acc.max(x.MaxX));
    let min_y = extents.iter().fold(f32::INFINITY, |acc, x| acc.min(x.MinY));
    let max_y = extents.iter().fold(f32::NEG_INFINITY, |acc, x| acc.max(x.MaxY));
    let new_extent = LayerExtent{MinX: min_x, MaxX: max_x, MinY: min_y, MaxY: max_y};
    debug!("new extent {:?} from extents: {:?}", new_extent, extents);

    let x_el = (min_x + max_x) / 2.0;
    let y_el = (min_y + max_y) / 2.0;
    let new_center = (x_el, y_el);
    debug!("new center: {:?}", new_center);

    let y_height = max_y - min_y;

    let view_height = view.size().y;
    debug!("View height: {view_height}");
    let y_zoom = (view_height / y_height).floor();
    debug!("y_height: {y_height}");
    debug!("y_zoom: {y_zoom}");
    (new_center, y_zoom)
}