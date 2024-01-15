use std::process::exit;

use env_logger;
use geometry::{get_plotgeo_from_layer, PlotGeometry};
use sfml::window::Event;
use sfml::window::Key;

mod transform;
mod geometry;
mod config;
mod render;

use crate::render::{create_window, render_loop};

fn main() {
    env_logger::init();
    let path_str = config::get_chart_path();
    let ds = geometry::get_dataset(&path_str[..]);
    // find the layer names we are interested in
    let layer_names = config::get_layers();
    // get the plotgeos for each layer
    let mut plotvec: Vec<PlotGeometry> = Vec::new();
    // get the extent from the config
    let (top_left_extent, bottom_right_extent) = config::get_init_tl_br();
    for layer_name in layer_names {
        let layer_color = config::get_color_for_layer(&layer_name[..]);
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
                Event::KeyPressed {code: Key::Escape, ..} => window.close(),
                Event::KeyPressed { code: Key::Up, ..} => {
                    zoom += 0.1;
                }
                Event::KeyPressed { code: Key::Down, ..} => {
                    zoom -= 0.1;
                }
                _ => {}
            }
        }
    render_loop(&mut window, &plotvec, zoom);
    }
}
