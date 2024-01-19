use std::process::exit;

use env_logger;

use log::info;
use sfml::graphics::RenderTarget;
use sfml::graphics::Color;
use sfml::window::Event;
use sfml::window::Key;

mod transform;
mod geometry;
mod config;
mod render;

use geometry::{get_plotgeo_from_layer_in_dataset, PlotGeometry, get_dataset, get_soundg_layer, get_soundg_coords, get_extent_from_layers_in_ds};
use config::{get_color_for_layer, get_resolution, get_layers, get_chart_directory, get_init_tl_br};
use render::{create_window, render_objects};

use gdal::Dataset;

use std::fs::read_dir;

use crate::geometry::DepthLayer;
// use log::debug;

fn main() {
    env_logger::init();
    // get the extent from the config
    //let (mut top_left_extent, mut bottom_right_extent) = get_init_tl_br();
    let mut top_left_extent = (f64::MIN, f64::MIN);
    let mut bottom_right_extent = (f64::MAX, f64::MAX);

    // get depth layer
    // print depth soundings
 
    let resolution = get_resolution();    
    // find the layer names we are interested in
    let layer_names = get_layers();
    // get the plotgeos for each layer
    let mut plotvec: Vec<PlotGeometry> = Vec::new();
    let chart_config_dir = get_chart_directory();
    let chart_dir = read_dir(chart_config_dir).unwrap();
    
    let mut plot_refs: Vec<PlotGeometry> = Vec::new();
    let depth_plots: Vec<DepthLayer> = Vec::new();

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
        //let mut soundg = get_soundg_layer(&ds);
        //let depth_sounding = get_soundg_coords(&mut soundg, (top_left_extent, bottom_right_extent));
        //depth_plots.push(epth_sounding);
        // update extent if layer's extent is smaller or larger
        let (br, tl) = get_extent_from_layers_in_ds(&layer_names, &ds);
        info!("Top left exent: {:?} Bottom right extent: {:?}", top_left_extent, bottom_right_extent);
        if tl.0 > top_left_extent.0 {
            top_left_extent.0 = tl.0;
            info!("Updating top left extent lon to {:.5}", top_left_extent.0);
            info!("Top left exent: {:?} Bottom right extent: {:?}", top_left_extent, bottom_right_extent);
        }
        if tl.1 > top_left_extent.1 {
            top_left_extent.1 = tl.1;
            info!("Updating top left extent lat to {:.5}", top_left_extent.1);
            info!("Top left exent: {:?} Bottom right extent: {:?}", top_left_extent, bottom_right_extent);
        }
        if br.0 < bottom_right_extent.0 {
            bottom_right_extent.0 = br.0;
            info!("Updating bottom right extent lon to {:.5}", bottom_right_extent.0);
            info!("Top left exent: {:?} Bottom right extent: {:?}", top_left_extent, bottom_right_extent);
        }
        if br.1 < bottom_right_extent.1 {
            bottom_right_extent.1 = br.1;
            info!("Updating bottom right extent lat to {:.5}", bottom_right_extent.1);
            info!("Top left exent: {:?} Bottom right extent: {:?}", top_left_extent, bottom_right_extent);
        }
        for layer_name in &layer_names {
            let layer_color = get_color_for_layer(&layer_name[..]);
            let plotgeo = get_plotgeo_from_layer_in_dataset(layer_name, &ds, layer_color);
            plotvec.push(plotgeo);
        }
    }
    for mut pg in plotvec {
    pg.triangulate_and_scale(top_left_extent, bottom_right_extent);
    plot_refs.push(pg);
    }
    //plot_refs.push(&soundg;
    // set up window and zoom
    println!("Creating Window!");
    let mut viewvec = (0.0 as f32, 0.0 as f32);
    let mut window = create_window();
    let mut zoom = 1.0 as f32;
    let zoom_scalar_x = resolution.0 as f32 / 10 as f32;
    let zoom_scalar_y = resolution.1 as f32 / 10 as f32;
    loop {
        while let Some(event) = window.wait_event() {
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
                    viewvec.1 += 1 as f32 / zoom as f32 * zoom_scalar_y;
                }
                Event::KeyPressed { code: Key::Down, ..} => {
                    viewvec.1 -= 1 as f32 / zoom as f32 * zoom_scalar_y;
                }
                Event::KeyPressed { code: Key::Left, ..} => {
                    viewvec.0 += 1 as f32 / zoom as f32 * zoom_scalar_x;
                }
                Event::KeyPressed { code: Key::Right, ..} => {
                    viewvec.0 -= 1 as f32 / zoom as f32 * zoom_scalar_x;
                }
                Event::KeyPressed { code: Key::W, ..} => {
                    zoom *= 1.1;
                }
                Event::KeyPressed { code: Key::S, ..} => {
                    zoom *= 0.9;
                }
                _ => {}
            }
        window.clear(Color::BLACK);
        render_objects(&mut window, &plot_refs, zoom, resolution, viewvec);
        render_objects(&mut window, &depth_plots, zoom, resolution, viewvec);
        window.display();
        }
    }
}
