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

use geometry::{get_plotgeo_from_layer_in_dataset, PlotGeometry, Plotable, get_dataset, get_soundg_layer, get_soundg_coords, get_extent_from_layers_in_ds};
use config::{get_color_for_layer, get_resolution, get_layers, get_chart_directory, get_init_tl_br};
use render::{create_window, render_objects};

use gdal::Dataset;

use std::fs::read_dir;
use log::debug;

fn main() {
    env_logger::init();
    // get the extent from the config
    let (top_left_extent, bottom_right_extent) = get_init_tl_br();
    
    // get depth layer
    // print depth soundings
 
    let resolution = get_resolution();    
    // find the layer names we are interested in
    let layer_names = get_layers();
    // get the plotgeos for each layer
    let mut plotvec: Vec<Box<PlotGeometry>> = Vec::new();
    let chart_config_dir = get_chart_directory();
    let chart_dir = read_dir(chart_config_dir).unwrap();
    
    let mut plot_refs: Vec<Box<dyn Plotable>> = Vec::new();
    let mut depth_plots: Vec<Box<dyn Plotable>> = Vec::new();

    let mut paths: Vec<String> = Vec::new();

    for entry in chart_dir {
        // append all paths to the vector paths
        let entry = entry.unwrap();
        let path = entry.path();
        let path = path.to_str().unwrap();
        paths.push(path.to_string());

    }
    let datasets: Vec<Dataset> = paths.iter().map(|path| get_dataset(path)).collect();
    for ds in datasets {
        let mut soundg = get_soundg_layer(&ds);
        let depth_sounding = get_soundg_coords(&mut soundg, (top_left_extent, bottom_right_extent));
        depth_plots.push(Box::new(depth_sounding));
        for layer_name in &layer_names {
            let layer_color = get_color_for_layer(&layer_name[..]);
            let plotgeo = get_plotgeo_from_layer_in_dataset(layer_name, &ds, layer_color);
            plotvec.push(Box::new(plotgeo));
        }
    }
    for mut pg in plotvec {
    pg.triangulate_and_scale(top_left_extent, bottom_right_extent);
    plot_refs.push(pg);
    }
    //plot_refs.push(&soundg;
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
    render_objects(&mut window, &plot_refs, zoom, resolution);
    render_objects(&mut window, &depth_plots, zoom, resolution);
    window.display();
    }
}
