#![allow(non_upper_case_globals)]
use std::vec;

use gdal::{Dataset, Metadata};
use gdal::version::VersionInfo;
use gdal::vector::{LayerAccess, OGRwkbGeometryType::*, Layer};

use geo::triangulate_spade::Triangles;
use geo::{Polygon, LineString, TriangulateEarcut, CoordsIter};

use log::{debug, info, warn};
use sfml::graphics::{Color, Font, RenderWindow, Vertex};

use crate::transform::mercator_transform;
use crate::render::{draw_vertex_vector, render_soundg};

use crate::config::get_merc_scaling_size;

use std::collections::HashMap;

pub trait Plotable {
    fn render(&self, window: &mut RenderWindow) -> ();

}
pub struct PlotGeometry{
    pub polygons: Vec<Polygon>,
    pub triangles: Triangles<f64>,
    pub color: sfml::graphics::Color,
    pub layer_name: String,
    pub vertex_vec: Vec<Vertex>,
}
impl PlotGeometry {
    pub fn new(polygons: Vec<Polygon>, triangles: Triangles<f64>, color: sfml::graphics::Color, layer_name: String, vertex_vec: Vec<Vertex>) -> PlotGeometry {
        PlotGeometry {
            polygons,
            triangles,
            color,
            layer_name,
            vertex_vec,
        }
    }
    pub fn triangulate_and_scale(&mut self) {
        self.triangles = triangles_from_scaled_polygons(&self.polygons);
        let mut vertex_vec: Vec<Vertex> = Vec::new();
        for  triangle in self.triangles.iter() {
            vertex_vec.push(Vertex::with_pos_color((triangle.0.x as f32, triangle.0.y as f32).into(), self.color));
            vertex_vec.push(Vertex::with_pos_color((triangle.1.x as f32, triangle.1.y as f32).into(), self.color));
            vertex_vec.push(Vertex::with_pos_color((triangle.2.x as f32, triangle.2.y as f32).into(), self.color));
        }
        self.vertex_vec = vertex_vec;
    }
}

impl Plotable for PlotGeometry {
    fn render(&self, window: &mut RenderWindow) {
        draw_vertex_vector(window, &self.vertex_vec)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerExtent {
    pub MinX: f32,
    pub MaxX: f32,
    pub MinY: f32,
    pub MaxY: f32,
}

// creates a PlotGeometry from a layer name - still needs to be triangulated and scaled
pub fn get_plotgeo_from_layer_in_dataset(layer_name: &String, ds: & Dataset, color: sfml::graphics::Color) -> PlotGeometry {
    let mut layers = get_layers(&ds, vec![&layer_name[..]]);
    let polygons = get_merc_polygons_from_layers(&mut layers);
    let triangles = Triangles::new();
    PlotGeometry::new(polygons, triangles, color, layer_name.clone(), Vec::new())
}

pub fn get_dataset(path: &str) -> Result<Dataset, gdal::errors::GdalError>{
    let ds = Dataset::open(path)?;
    let layer_count = ds.layer_count();
    let has_geo = VersionInfo::has_geos();
    debug!("Dataset Description: {}", ds.description().unwrap());
    debug!("Has GEOS: {} Layer Count: {}", has_geo, layer_count);
    Ok(ds)
}

pub fn get_layers<'a> (ds: &'a Dataset, names: Vec<&'a str>) -> Vec<Layer <'a>> {
    let layer_count = ds.layer_count();
    let mut layers: Vec<Layer> = Vec::new();
    debug!("Getting layers!");
    for l in 0..layer_count {
        let layer = ds.layer(l).unwrap();
        let layer_name = layer.name().clone();
        if names.contains(&&layer_name[..]) {
            layers.push(layer);
            info!("Added layer: {}", layer_name)
        }
    }
    layers
}

pub fn triangles_from_scaled_polygons(polygons: &Vec<Polygon>) -> Triangles<f64> {
    let mut triangles: Triangles<f64> = Vec::new();
    for poly in polygons {
        let scaled_coords: Vec<(f64, f64)> = poly.exterior().coords_iter().map(|x: geo_types::Coord| (x.x, x.y)).collect();
        let new_poly = Polygon::new(LineString::from(scaled_coords), vec![]);
        if new_poly.exterior().coords_count() < 4 {
            warn!("Polygon has less than 4 points! Continuing..");
            warn!("Polygon before transformation had {} points", poly.exterior().coords_count());
            continue;
        }
        let scaled_triangles = new_poly.earcut_triangles();
        //debug!("Found {} triangles", scaled_triangles.len());
        for scaled_triangle in scaled_triangles {
            triangles.push(scaled_triangle);
        }
    }
    triangles
}

/// performs a mercator transform on all the geometries in a layer
pub fn get_merc_polygons_from_layers(layers: &mut Vec<gdal::vector::Layer>) -> Vec<Polygon> {
    let mut polygons: Vec<Polygon> = Vec::new();
    let merc_scale = get_merc_scaling_size();
    debug!("get_polygons_from_layers called! with {} layers", layers.len());
    for layer in layers {
        let layer_name = layer.name().clone();
        debug!("Checking layer {}", layer_name);
        for feature in layer.features()
        {
            let geometry = match feature.geometry() {
                Some(geo) => geo,
                None => {
                    debug!("[get_extent_from_layers] {} has no geometry!", layer_name);
                    continue;
                }
            };
            let geom_type_name = geometry.geometry_name();
            let geo_count = geometry.geometry_count();
            debug!("{} has {} geometries", layer_name, geo_count);
            for i in 0..geo_count{
                let new_geo = geometry.get_geometry(i);
                let new_geo_name = new_geo.geometry_name();
                match new_geo.geometry_type() {
                    wkbLinearRing | wkbLineString | wkbPolygon => {
                        debug!("Matched {new_geo_name}");
                        let points = new_geo.get_point_vec();
                        let num_points = points.len();
                        let merc_points_2d: Vec<(f64, f64)> = points.iter().map(|x| (x.0, x.1)).map(|x| mercator_transform(x, merc_scale)).collect();
                        let poly = Polygon::new(LineString::from(merc_points_2d), vec![]);
                        let num_poly_points = poly.exterior().coords_count();
                        polygons.push(poly);
                        debug!("Added a new geometry to the list with {} points before transform {}", num_poly_points, num_points);
                    },
                    unsure => {
                        debug!("{geom_type_name} is {unsure}");
                    }
                }
            }
        }
    }
    polygons
}

fn get_color_for_depth(depth: f64) -> Color {
    // 19 deep
    // 9.5 safe
    // 6.5 shallow
    if depth > 19.0 {
        return Color::rgb(212,234,238)
    }
    else if depth > 6.5 {
        return Color::rgb(186,213,225)
    }
    else if depth <= 6.5 {
        return Color::rgb(115,182,239)
    }
    Color::BLACK
}


pub fn get_depare_from_layer(layer: &mut gdal::vector::Layer) -> DEPARE {
    let mut depare_layers: Vec<DepareLayer> = Vec::new();
    let merc_scale = get_merc_scaling_size();
    let layer_name = layer.name().clone();
    let mut vertex_vec = Vec::new();
    let extent: LayerExtent = LayerExtent { MinX: f32::MAX, MaxX: f32::MIN, MinY: f32::MAX, MaxY: f32::MIN };
    debug!("Checking layer {}", layer_name);
    for feature in layer.features()
    {
        let geometry = match feature.geometry() {
            Some(geo) => geo,
            None => {
                debug!("[get_extent_from_layers] {} has no geometry!", layer_name);
                continue;
            }
        };
        let min_value = match feature.field_as_double_by_name("DRVAL1") {
            Ok(val) => {
                match val {
                    Some(val) => val,
                    None => {
                        warn!("No DRVALUE1 field found in DEPARE layer!");
                        continue;
                    }
                }
            }
            Err(_) => {
                warn!("No DRVALUE1 field found in DEPARE layer!");
                continue;
            }
        };
        let max_value = match feature.field_as_double_by_name("DRVAL2") {
            Ok(val) => {
                match val {
                    Some(val) => val,
                    None => {
                        warn!("No DRVALUE2 field found in DEPARE layer!");
                        continue;
                    }
                }
            }
            Err(_) => {
                warn!("No DRVALUE2 field found in DEPARE layer!");
                continue;
            }
        };

        let geom_type_name = geometry.geometry_name();
        let geo_count = geometry.geometry_count();
        debug!("{} has {} geometries", layer_name, geo_count);
        for i in 0..geo_count{
            let new_geo = geometry.get_geometry(i);
            let new_geo_name = new_geo.geometry_name();
            match new_geo.geometry_type() {
                wkbLinearRing | wkbLineString | wkbPolygon => {
                    debug!("Matched {new_geo_name}");
                    let points = new_geo.get_point_vec();
                    let num_points = points.len();
                    
                    // handle the points
                    let merc_points_2d: Vec<(f64, f64)> = points.iter().map(|x| (x.0, x.1)).map(|x| mercator_transform(x, merc_scale)).collect();

                    let poly = Polygon::new(LineString::from(merc_points_2d), vec![]);
                    let num_poly_points = poly.exterior().coords_count();
                    let polygon_vec = vec![poly];
                    // todo take care of case with 3 points
                    if num_poly_points < 4 {
                        warn!("Polygon has less than 4 points! Continuing..");
                        continue;
                    }
                    let color: Color;
                    // convert m to f
                    let foot_depth = min_value * 3.281;
                    color = get_color_for_depth(foot_depth); 
                    let poly_triangles = triangles_from_scaled_polygons(&polygon_vec);
                    for  triangle in poly_triangles.iter() {
                        vertex_vec.push(Vertex::with_pos_color((triangle.0.x as f32, triangle.0.y as f32).into(), color));
                        vertex_vec.push(Vertex::with_pos_color((triangle.1.x as f32, triangle.1.y as f32).into(), color));
                        vertex_vec.push(Vertex::with_pos_color((triangle.2.x as f32, triangle.2.y as f32).into(), color));
                    }

                    let color = Color::WHITE;
                    let depth = (min_value, max_value);
                    let depare_layer = DepareLayer::new(vec![], color, depth);
                    depare_layers.push(depare_layer);
                    debug!("Added a new geometry to the list with {} points before transform {} with depth ranging from {min_value}-{max_value}m", num_poly_points, num_points);
                },
                unsure => {
                    debug!("{geom_type_name} is {unsure}");
                }
            }
        }
    }
    let mut d = DEPARE { layers: depare_layers, vertices: vertex_vec, extent: extent };
    d.sum_vertices();
    let extent = get_vertices_extent(&d.vertices);
    d.extent = extent;
    d
}

#[derive(Clone, Debug)]
pub struct DEPARE {
    pub layers: Vec<DepareLayer>,
    pub vertices: Vec<Vertex>,
    pub extent: LayerExtent,
}

impl DEPARE {
    pub fn sum_vertices(&mut self) {
        for layer in &self.layers {
            for vertex in &layer.vertices {
                self.vertices.push(*vertex);
            }
        }
    }
}

impl Plotable for DEPARE {
    fn render(&self, window: &mut RenderWindow) -> () {
        draw_vertex_vector(window, &self.vertices)
    }
}
#[derive(Clone, Debug)]
pub struct DepareLayer {
    pub vertices: Vec<Vertex>,
    pub color: Color,
    pub depth: (f64, f64),
}

impl DepareLayer {
    pub fn new(vertices: Vec<Vertex>, color: Color, depth: (f64, f64)) -> DepareLayer {
        DepareLayer { vertices, color, depth }
    }
}

pub struct DepthLayer {
    pub coordinates: Vec<(f64, f64, f64)>,
    pub font: sfml::SfBox<Font>,
    pub longitude_scale: (f64, f64),
    pub latitude_scale: (f64, f64),
}

impl DepthLayer {
    pub fn project_coords(&mut self) {
        let merc_scale = get_merc_scaling_size();
        let mut final_points: Vec<(f64, f64, f64)> = Vec::new();
        for point in &self.coordinates {
            let merc_point = mercator_transform((point.0, point.1), merc_scale);
            let (x, y) = merc_point;
            final_points.push((x, y, point.2));
            if x < self.longitude_scale.0 {
                self.longitude_scale.0 = x;
            }
            if x > self.longitude_scale.1 {
                self.longitude_scale.1 = x;
            }
            if y < self.latitude_scale.0 {
                self.latitude_scale.0 = y;
            }
            if y > self.latitude_scale.1 {
                self.latitude_scale.1 = y;
            }
        }
        self.coordinates = final_points;
    }
}

impl Plotable for DepthLayer {
    fn render(&self, window: &mut RenderWindow) -> () {
        render_soundg(window, self, &self.font, 0.0 as f32)
    }
    
}
pub fn get_soundg_layer(ds: & Dataset) -> Option<Layer> {
    let mut layers = get_layers(&ds, vec!["SOUNDG"]);
    let layer = layers.pop();
    match layer {
        Some(layer) => Some(layer),
        None => {
            warn!("No SOUNDG layer found in dataset!");
            None
        }
    }
}

pub fn get_depare_layer(ds: & Dataset) -> Option<Layer> {
    let mut layers = get_layers(&ds, vec!["DEPARE"]);
    let layer = layers.pop();
    match layer {
        Some(layer) => Some(layer),
        None => {
            warn!("No DEPARE layer found in dataset!");
            None
        }
    }
}

pub fn get_soundg_coords(soundg_layer: &mut Layer) -> DepthLayer {
    let mut final_points: Vec<(f64, f64, f64)> = Vec::new();
    for feature in soundg_layer.features() {
        let geometry = match feature.geometry() {
            Some(geo) => geo,
            None => {
                debug!("soundg has no geometry!");
                continue;
            }
        };
        let geom_type_name = geometry.geometry_name();
        let geo_count = geometry.geometry_count();
        debug!("soundg has {} geometries", geo_count);
        for i in 0..geo_count{
            let new_geo = geometry.get_geometry(i);
            match new_geo.geometry_type() {
                wkbMultiPointZM | wkbMultiPoint25D | wkbPoint25D => {
                    let points = new_geo.get_point_vec();
                    for point in points {
                        final_points.push((point.0, point.1, point.2));

                    }
                }, 
                unsure => {
                    debug!("{geom_type_name} is {unsure}");
                }
            }
        }
    }
    DepthLayer { coordinates: final_points, font: get_default_font(), longitude_scale: (f64::MAX, f64::MIN), latitude_scale: (f64::MAX, f64::MIN)} 
} 



pub fn get_default_font() -> sfml::SfBox<Font> {
    Font::from_file("./src/fonts/OpenSans-Regular.ttf").unwrap() 
}

fn does_envelope_collide(extent1: &LayerExtent, extent2: &LayerExtent) -> bool {
    // return true if envelope 2 collides with envelope 1
    // if e2 is higher than e1
    // need to check this function
    debug!("Checking if {:?} collides with {:?}", extent1, extent2);
    // if e2 is above e1
    if extent2.MinY > extent1.MaxY {
        debug!("No collision! {:?} is above {:?}", extent2, extent1);
        return false
    }
    // if e2 is lower than e1
    if extent2.MaxY < extent1.MinY {
        debug!("No collision! {:?} is below {:?}", extent2, extent1);
        return false
    }
    if extent2.MaxX < extent1.MinX {
    // if e2 is to the left of e1
        debug!("No collision! {:?} is to the left of {:?}", extent2, extent1);
        return false
    }
    if extent2.MinX > extent1.MaxX {
    // if e2 is to the right of e1
        debug!("No collision! {:?} is to the right of {:?}", extent2, extent1);
        return false
    }
    debug!("COLLISION! {:?} collides with {:?}", extent1, extent2);
    true
}

pub fn get_vertices_extent(vertices: &Vec<Vertex>) -> LayerExtent {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for vertex in vertices {
        let pos = vertex.position;
        let x = pos.x;
        let y = pos.y;
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
    }
    LayerExtent { MinX: min_x, MaxX: max_x, MinY: min_y, MaxY: max_y }
}

pub fn find_collisions(layer_map: &HashMap<String, LayerExtent>) -> Vec<Vec<String>> {
    let mut cols = Vec::new();
    let layer_names = layer_map.keys().collect::<Vec<&String>>();

    let mut checked: Vec<String> = Vec::new();
    let mut stack: Vec<String> = Vec::new();

    let mut to_check = layer_names.clone();
    while to_check.len() >= 1 {
        let layer_name = to_check.pop().unwrap();
        if checked.contains(&layer_name) {
            continue;
        }
        stack.push(layer_name.clone());
        debug!("Starting with {}", layer_name);

        let mut collisions: Vec<String> = Vec::new();
        while stack.len() >= 1 {
            let layer_name = stack.pop().unwrap();
            debug!("Checking layer {}", layer_name);
            debug!("Stack = {:?}", stack.clone());
            let extent = layer_map.get(&layer_name).unwrap();
            let mut curr_collisions = Vec::new();
            for other_layer in layer_names.clone() {
                if other_layer == &layer_name {
                    continue;
                }
                if checked.contains(&other_layer) {
                    continue;
                }

                let other_extent = layer_map.get(other_layer).unwrap();
                debug!("Checking if {} and {} collide", &layer_name, &other_layer);
                if does_envelope_collide(&extent, &other_extent) {
                    curr_collisions.push(other_layer.clone());
                    if !collisions.contains(&layer_name) {
                        collisions.push(layer_name.clone());
                    }
                    if !collisions.contains(other_layer) {
                        collisions.push(other_layer.clone());
                    }
                    if !stack.contains(&other_layer) {
                        debug!("Pushing {} onto stack", other_layer);
                        stack.push(other_layer.clone());
                    }
                }
            }
            debug!("Collisions for {} are {:?}", layer_name, curr_collisions);
            checked.push(layer_name.clone());

        }
        
        if collisions.len() > 0 {
            debug!("Adding collisions to list! {:?}", collisions.clone());
            cols.push(collisions);
        }

    } 
    cols
}

pub fn get_extent_area(extent: &LayerExtent) -> f32 {
    debug!("Getting area for {:?}", &extent);
    let x = extent.MaxX - extent.MinX;
    let y = extent.MaxY - extent.MinY;
    x.abs() * y.abs() 
}

pub fn get_hashmap_of_depare_layers(depare_layers: &mut Vec<(DEPARE, String)>) -> HashMap<u16, Vec<DEPARE>> {
    // create map of datasets
    let mut map: HashMap<u16, Vec<DEPARE>>  = HashMap::new();
    let depare_copy = depare_layers.clone();
    let mut name2extent: HashMap<String, LayerExtent> = HashMap::new();
    
    for (depare_layer, s) in depare_layers {
        name2extent.insert(s.clone(), depare_layer.extent.clone());
        }
    debug!("Looking at Map!");
    for k in name2extent.keys() {
        let v = name2extent.get(k).unwrap();
        debug!("{k}: {:?}", v);
    }
    // find collisions
    let collisions = find_collisions(&name2extent);
    let mut total_cols = Vec::new();
    for vc in collisions.clone() {
        for c in vc {
            total_cols.push(c.clone());
        }
    }
    debug!("Collisions: {:?}", collisions.clone()); 
    // sort collisions by area - maybe should make it more centroidy
    let mut cntr = 1;
    for col_vec in collisions {
        let mut sort_vec = col_vec.clone();
        debug!("Sorting {:?}", sort_vec.clone());
        sort_vec.sort_by(|a, b| {
        let a_extent = name2extent.get(a).unwrap();
        let b_extent = name2extent.get(b).unwrap();
        let a_area = get_extent_area(a_extent);
        let b_area = get_extent_area(b_extent);
        b_area.partial_cmp(&a_area).unwrap() // sort it backwards - so smallest are rendered last
        });
        debug!("Sorted Vec: {:?}", sort_vec);
        let depare_vec: Vec<DEPARE> = sort_vec.iter().map(|x: &String| {
            let layer = depare_copy.iter().find(|(_depare, s)| s == x).unwrap();
            layer.0.clone()
        }).collect();
        map.insert(cntr, depare_vec);
        cntr += 1
    }
    let mut collision_free_vec = Vec::new();
    for layer_name in depare_copy {
        if !total_cols.contains(&layer_name.1) {
            collision_free_vec.push(layer_name.0);
        }
    }
    map.insert(0, collision_free_vec);
    map
}