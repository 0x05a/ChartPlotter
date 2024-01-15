use env_logger;
use geometry::{get_plotgeo_from_layer, PlotGeometry};
use sfml::graphics::{Color, RenderTarget};
use sfml::window::Event;
use sfml::window::Key;

mod transform;
mod geometry;
mod config;
mod render;


// use PlotGeo to render instead of using all these functions

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
    let mut window = render::create_window();
    let mut zoom = 1.0 as f32;
    window.set_framerate_limit(60);
    loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
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
        window.clear(Color::BLACK);
        // render code
        for plotgeo in &plotvec {
            plotgeo.render(&mut window, zoom)
        }
        window.display();
        }
}


    //let interesting_layer_names = config::get_layers();
    //let mut seaare = get_plotgeo_from_layer("SEAARE".to_string(), ds, Color::BLUE);
    //let (top_left_extent, bottom_right_extent) = config::get_init_tl_br();
    //seaare.triangulate_and_scale(top_left_extent, bottom_right_extent);
    //let l_color = config::get_color_for_layer("LNDARE");
    //dbg!(l_color);
    //let mut layers = geometry::get_layers(&ds, interesting_layer_names);
    //let polygons = geometry::get_merc_polygons_from_layers(&mut layers);
    //let (top_left_extert, bottom_right_extent) = config::get_init_tl_br();
    //let triangles = geometry::triangles_from_scaled_polygons(&polygons, (top_left_extert, bottom_right_extent));


// open ds
// get config layers
// get layers from ds
// get geometries for layers
// convert geometries to mercator coords
// convert mercator coords to cartesian coords
// triangulate cartesian coords
// draw triangles




//    let ds = geometry::get_dataset("/home/zack/chart_plotter/rust_rewrite/chartplotter/src/charts/BYU_CHICO.000");
//    let interesting_layer_names = config::get_layers();
//    let layers = geometry::get_layers(&ds, interesting_layer_names);
//    for layer in layers.into_iter()
//    {
//        let mut layer = layer;
//        let layer_name = layer.name();
//        let geometries = geometry::get_ChartGeomentry_from_layer(&mut layer);
//        println!("Layer {} has {} geometries: ", layer_name, geometries.len());
//        for geo in geometries {
//            // print number of coords and triangles
//            println!("\t {} coords and {} triangles", geo.geo.exterior().coords_count(), geo.triangles.len());
//        }
//    }