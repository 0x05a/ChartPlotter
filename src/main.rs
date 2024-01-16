use std::process::exit;

use env_logger;

use sfml::graphics::RenderTarget;
use sfml::graphics::Color;
use sfml::window::Event;
use sfml::window::Key;

mod transform;
mod geometry;
mod config;
mod render;

use geometry::{get_plotgeo_from_layer, PlotGeometry, get_dataset, get_soundg_layer, get_soundg_coords};
use config::{get_color_for_layer, get_resolution, get_layers, get_chart_path, get_init_tl_br};
use render::{create_window, render_poly, render_soundg};

fn main() {
    env_logger::init();
    let path_str = get_chart_path();
    let ds = get_dataset(&path_str[..]);
    // get the extent from the config
    let (top_left_extent, bottom_right_extent) = get_init_tl_br();
    
    // get depth layer
    let mut soundg = get_soundg_layer(&ds);
    let depth_soundings = get_soundg_coords(&mut soundg, (top_left_extent, bottom_right_extent));
    // print depth soundings
 
    let resolution = get_resolution();    
    // find the layer names we are interested in
    let layer_names = get_layers();
    // get the plotgeos for each layer
    let mut plotvec: Vec<PlotGeometry> = Vec::new();
    for layer_name in layer_names {
        let layer_color = get_color_for_layer(&layer_name[..]);
        let mut plotgeo = get_plotgeo_from_layer(layer_name, &ds, layer_color);
        plotgeo.triangulate_and_scale(top_left_extent, bottom_right_extent);
        plotvec.push(plotgeo);

    }
    // set up window and zoom
    println!("Creating Window!");
    let mut window = create_window();
    let mut zoom = 1.0 as f32;
    window.set_framerate_limit(10);
    loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => {
                    window.close();
                    exit(0);
                }
                Event::KeyPressed {code: Key::Escape, ..} => {
                    window.close();
                    exit(0);
                }
                Event::KeyPressed { code: Key::Up, ..} => {
                    zoom += 0.1;
                }
                Event::KeyPressed { code: Key::Down, ..} => {
                    zoom -= 0.1;
                }
                _ => {}
            }
        }
    window.clear(Color::BLACK);
    render_poly(&mut window, &plotvec, zoom);
    render_soundg(&mut window, &depth_soundings, resolution, zoom);
    window.display();
    }
}
