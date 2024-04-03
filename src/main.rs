use std::process::exit;

use env_logger;

use sfml::graphics::RenderTarget;
use sfml::graphics::Color;
use sfml::graphics::View;
use sfml::window::Event;
use sfml::window::Key;

mod transform;
mod geometry;
mod config;
mod render;

use geometry::{get_plotgeo_from_layer_in_dataset, PlotGeometry, get_dataset, get_soundg_layer, get_soundg_coords, DEPARE, Plotable, get_hashmap_of_depare_layers};
use config::{get_color_for_layer, get_resolution, get_layers, get_chart_directory};
use render::{create_window, render_objects};

use gdal::Dataset;

use std::fs::read_dir;
use crate::geometry::get_depare_from_layer;
use crate::geometry::get_depare_layer;
use crate::geometry::DepthLayer;
use crate::render::get_zoom;
use log::info;

fn main() {
    env_logger::init();
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
    let mut depth_plots: Vec<DepthLayer> = Vec::new();
    let mut projections: Vec<DepthLayer> = Vec::new();

    let mut paths: Vec<String> = Vec::new();

    for entry in chart_dir {
        // append all paths to the vector paths
        let entry = entry.unwrap();
        let path = entry.path();
        let path = path.to_str().unwrap();
        paths.push(path.to_string());

    }
    let mut datasets: Vec<(Dataset, String)> = Vec::new();
    for path in paths {
        let ds = get_dataset(&path);
        match ds {
            Ok(ds) => datasets.push((ds, path)),
            _ => continue,
        }
    }
    //let datasets: Vec<Dataset> = paths.iter().map(|path| get_dataset(path)).map(|ds| ds.unwrapor
    let mut resolve_depare = Vec::new();
    for (ds, p) in datasets {
        for layer_name in &layer_names {
            let layer_color = get_color_for_layer(&layer_name[..]);
            let plotgeo = get_plotgeo_from_layer_in_dataset(layer_name, &ds, layer_color);
            plotvec.push(plotgeo);
        }
        let soundg = get_soundg_layer(&ds);
        let mut soundg_layer = match soundg {
            Some(soundg) => soundg,
            _ => continue,
        };
        let depth_sounding = get_soundg_coords(&mut soundg_layer);
        depth_plots.push(depth_sounding);
        // update extent if layer's extent is smaller or larger
        let mut depare_layer = match get_depare_layer(&ds) {
            Some(depare) => depare,
            _ => continue,
        };
        let depare: DEPARE = get_depare_from_layer(&mut depare_layer);
        resolve_depare.push((depare, p.clone()));
        }

    info!("Handling DEPARE RESOLVING!");
    let map = get_hashmap_of_depare_layers(&mut resolve_depare);
    info!("map: {:?}", map.keys());


    for mut pg in plotvec {
    pg.triangulate_and_scale();
    plot_refs.push(pg);
    }
    for mut ds in depth_plots {
        ds.project_coords();
        projections.push(ds);
    }
    // set up window and zoom
    println!("Creating Window!");
    let mut window = create_window();
    let mut view = View::new((resolution.0 as f32 / 2 as f32, resolution.1 as f32 / 2 as f32).into(), (resolution.0 as f32, resolution.1 as f32).into());
    window.set_view(&view);
    let (center, zoom_scalar) = get_zoom(&map, &view);
    window.set_view(&view);
    let mut zoom = 1.0 as f32;
    let res_x = resolution.0 as f32;
    let res_y = resolution.1 as f32;
    let mid_x = res_x / 2.0;
    let mid_y = res_y / 2.0;
    info!("center: {:?}", center);
    view.move_((center.0 - mid_x, center.1 - mid_y));
    view.zoom(1.0 / zoom_scalar);
    zoom /= zoom_scalar;
    let mut render_depth = false;
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
                Event::KeyPressed { code: Key::Q, ..} => {
                    window.close();
                    exit(0);
                }
                Event::KeyPressed { code: Key::Up, ..} => {
                    view.move_((0 as f32, -0.1 * res_y * zoom as f32 ));
                }
                Event::KeyPressed { code: Key::Down, ..} => {
                    view.move_((0 as f32, 0.1  * res_y * zoom as f32 ));
                }
                Event::KeyPressed { code: Key::Left, ..} => {
                    view.move_((-0.1 * (res_x * zoom) as f32, 0 as f32));
                }
                Event::KeyPressed { code: Key::Right, ..} => {
                    view.move_((0.1 * (res_x * zoom) as f32 , 0 as f32));
                }
                Event::KeyPressed { code: Key::W, ..} => {
                    zoom *= 0.9;
                    view.zoom(0.9);
                    info!("view: {:?}", view.size());
                }
                Event::KeyPressed { code: Key::S, ..} => {
                    zoom *= 1.1;
                    view.zoom(1.1);
                    info!("view: {:?}", view.size());
                }
                Event::KeyPressed { code: Key::D, ..} => {
                    render_depth = !render_depth;
                }
                _ => {}
            }
        window.clear(Color::BLACK);

        render_objects(&mut window, &plot_refs);

        for key in map.keys() {
            let layers = map.get(key).unwrap();
            for depare in layers {
                depare.render(&mut window)
            }
        }       
        if render_depth {
            //render_objects(&mut window, &projections);
            for projection in projections.iter() {
                render::render_soundg(&mut window, projection, &projection.font, zoom);
            }
        }
        window.set_view(&view);
        window.display();
        }
    }
}
