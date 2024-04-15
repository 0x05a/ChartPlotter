use sfml::graphics::{CircleShape, Color, Font, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Shape, Transformable, Vertex, View};
use sfml::window::Style;
use log::debug;
use sfml::SfBox;

use crate::config;
use crate::geometry::{does_extent_collide, get_extent_area, BuoyLayer, DepthLayer, LayerExtent, Plotable, DEPARE};

use std::collections::HashMap;

pub fn create_window() -> RenderWindow {
    debug!("Creating window");
    let resolution = config::get_resolution(); 
    let window = RenderWindow::new(resolution, "SFML window", Style::NONE, &Default::default());
    window

}

pub fn render_objects<T: Plotable>(window: &mut RenderWindow, plotvec: &Vec<T>, window_view: &SfBox<View>) {

        // render code
        for plot in plotvec {
            plot.render(window, window_view)
        }
}

//pub fn draw_vertex_buffer(window: &mut RenderWindow, vertex_buffer: &VertexBuffer) {
//    info!("Drawing {} Vertices!", vertex_buffer.vertex_count());
//    window.draw(vertex_buffer);
//}

pub fn is_extent_in_view(view: &View, extent: &LayerExtent) -> bool {
    // if the extent is not in the view then return false
    let view_center = view.center();
    let view_size = view.size();
    let view_extent = LayerExtent{MinX: view_center.x - view_size.x / 2.0, MaxX: view_center.x + view_size.x / 2.0, MinY: view_center.y - view_size.y / 2.0, MaxY: view_center.y + view_size.y / 2.0};
    // info!("Checking if extent {:?} is in view {:?}", extent, view_extent);
    does_extent_collide(&view_extent, extent)
}

pub fn draw_vertex_vector(window: &mut RenderWindow, vertices: &Vec<Vertex>, vertex_extent: &LayerExtent, view: &SfBox<View>) {
    if !is_extent_in_view(view, vertex_extent) {
        // info!("Extent not in view!");
        return;
    }
    window.draw_primitives(vertices, PrimitiveType::TRIANGLES, &RenderStates::default());

}

pub fn render_soundg(window: &mut RenderWindow, depth_soundings: &DepthLayer,  font: &SfBox<Font>, scale: f32, view: &SfBox<View>) {
    let view_extent = LayerExtent{MinX: view.center().x - view.size().x / 2.0, MaxX: view.center().x + view.size().x / 2.0, MinY: view.center().y - view.size().y / 2.0, MaxY: view.center().y + view.size().y / 2.0};
    if !does_extent_collide(&view_extent, &depth_soundings.extent) {
        // info!("Extent not in view!");
        return;
    }
    let view_area = get_extent_area( &view_extent);
    let soundg_area = get_extent_area(&depth_soundings.extent);

    let soundg_scale = view_area / soundg_area;
    // log::info!("Soundg scale: {}", soundg_scale);
    if soundg_scale > 10.0 {
        return;
    }


    for sounding in &depth_soundings.coordinates {
        let mut text = sfml::graphics::Text::new(&format!("{:.0}", sounding.2 * 3.281), &font, 8);
        
        let pos = (sounding.0 as f32, sounding.1 as f32);
        
        text.set_position(pos);
        text.set_fill_color(Color::WHITE);
        text.scale((2.0 * scale, 2.0 * scale));
        window.draw(&text);
    }
}

pub fn render_buoy(window: &mut RenderWindow, buoy: &BuoyLayer, scale: f32, view: &SfBox<View>) {
    let view_extent = LayerExtent{MinX: view.center().x - view.size().x / 2.0, MaxX: view.center().x + view.size().x / 2.0, MinY: view.center().y - view.size().y / 2.0, MaxY: view.center().y + view.size().y / 2.0};
    //info!("View extent: {:?}", view_extent);
    //info!("Buoy extent: {:?}", buoy.extent);
    //if !does_extent_collide(&view_extent, &buoy.extent) {
        //info!("Extent not in view!");
        //return;
    //}
    let view_area = get_extent_area( &view_extent);
    let buoy_area = get_extent_area(&buoy.extent);

    let buoy_scale = view_area / buoy_area;
    // log::info!("Buoy scale: {}", buoy_scale);
    if buoy_scale > 40.0 {
        return;
    }
    for vertex in &buoy.vertices {
        
        let mut buoy_circle = CircleShape::new(5.0 * scale, 30); 
        buoy_circle.set_position(vertex.position);
        buoy_circle.set_fill_color(vertex.color);
        window.draw(&buoy_circle);
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