#![allow(non_upper_case_globals)]
use gdal::{Dataset, Metadata};
use gdal::version::VersionInfo;
use gdal::vector::{LayerAccess, OGRwkbGeometryType::*, Layer};


use geo::triangulate_spade::Triangles;
use geo::{Polygon, LineString, TriangulateEarcut, CoordsIter};

use log::{debug, info, warn};
use sfml::graphics::{Font, RenderWindow};
use sfml::graphics::{VertexBuffer, Vertex, PrimitiveType, Color, VertexBufferUsage};
use sfml::system::{Vector2, Vector2f};

use crate::transform::{mercator_transform, merc_to_cartesian_coords};
use crate::render::{draw_triangles, draw_vertex_buffer, draw_vertex_vector, render_soundg};

use crate::config::get_merc_scaling_size;


pub trait Plotable {
    fn render(&self, window: &mut RenderWindow, zoom: f32, viewvec: (f32, f32), resolution: (u32, u32)) -> ();

}
pub struct PlotGeometry{
    pub polygons: Vec<Polygon>,
    pub triangles: Triangles<f64>,
    pub color: sfml::graphics::Color,
    pub layer_name: String,
    pub vertex_vec: Vec<Vertex>,
    pub vertex_buff: VertexBuffer
}
impl PlotGeometry {
    pub fn new(polygons: Vec<Polygon>, triangles: Triangles<f64>, color: sfml::graphics::Color, layer_name: String, vertex_vec: Vec<Vertex>, vertex_buff: VertexBuffer) -> PlotGeometry {
        PlotGeometry {
            polygons,
            triangles,
            color,
            layer_name,
            vertex_vec,
            vertex_buff,
        }
    }
    pub fn triangulate_and_scale(&mut self, top_left: (f64, f64), bottom_right: (f64, f64)) {
        self.triangles = triangles_from_scaled_polygons(&self.polygons, (top_left, bottom_right));
        let mut vertex_buffer = VertexBuffer::new(PrimitiveType::TRIANGLES, self.triangles.len() as u32 * 3 as u32, VertexBufferUsage::STATIC);
        let mut vertex_vec: Vec<Vertex> = Vec::new();
        for  triangle in self.triangles.iter() {
            //vertex_vec[idx].position   = Vector2f::new(triangle.0.x as f32, triangle.0.y as f32);
            //vertex_vec[idx+1].position = Vector2f::new(triangle.1.x as f32, triangle.1.y as f32);
            //vertex_vec[idx+2].position = Vector2f::new(triangle.2.x as f32, triangle.2.y as f32);
            vertex_vec.push(Vertex::with_pos_color((triangle.0.x as f32, triangle.0.y as f32).into(), self.color));
            vertex_vec.push(Vertex::with_pos_color((triangle.1.x as f32, triangle.1.y as f32).into(), self.color));
            vertex_vec.push(Vertex::with_pos_color((triangle.2.x as f32, triangle.2.y as f32).into(), self.color));
        }

        // dbg!(&vertex_vec);
        assert!(vertex_vec.len() / 3 == self.triangles.len());
        let good_update = vertex_buffer.update(&self.vertex_vec.as_slice(), 0);
        if !good_update {
            info!("Could not update vertex buffer!!!!")
        }
        else {
            info!("Updated vertex buffer!!!")
        }
        debug!("Created vertex buffer with {} vertices on {} triangles", vertex_buffer.vertex_count(), self.triangles.len());
        self.vertex_buff = vertex_buffer;
        self.vertex_vec = vertex_vec;
    }
}

impl Plotable for PlotGeometry {
    fn render(&self, window: &mut RenderWindow, zoom: f32, viewvec: (f32, f32), resolution: (u32, u32)) {
        //draw_triangles(window, &self.triangles, zoom, resolution, viewvec, Some(self.color));
        //draw_vertex_buffer(window, &self.vertex_buff);
        draw_vertex_vector(window, &self.vertex_vec)
    }
}

// creates a PlotGeometry from a layer name - still needs to be triangulated and scaled
pub fn get_plotgeo_from_layer_in_dataset(layer_name: &String, ds: & Dataset, color: sfml::graphics::Color) -> PlotGeometry {
    let mut layers = get_layers(&ds, vec![&layer_name[..]]);
    let polygons = get_merc_polygons_from_layers(&mut layers);
    let triangles = Triangles::new();
    PlotGeometry::new(polygons, triangles, color, layer_name.clone(), Vec::new(), VertexBuffer::new(PrimitiveType::TRIANGLES, 0, VertexBufferUsage::DYNAMIC))
}

pub fn get_dataset(path: &str) -> Dataset {
    let ds = Dataset::open(path).unwrap();
    let layer_count = ds.layer_count();
    let has_geo = VersionInfo::has_geos();
    debug!("Dataset Description: {}", ds.description().unwrap());
    debug!("Has GEOS: {} Layer Count: {}", has_geo, layer_count);
    ds
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

pub fn triangles_from_scaled_polygons(polygons: &Vec<Polygon>, tl_br: ((f64, f64), (f64, f64))) -> Triangles<f64> {
    let mut triangles: Triangles<f64> = Vec::new();
    let merc_scale = get_merc_scaling_size();
    //debug!("triangles_from_scaled_polygons called!");
    //debug!("starting with {} polygons", polygons.len());
    for poly in polygons {
        let scaled_coords: Vec<(f64, f64)> = poly.exterior().coords_iter().map(|x: geo_types::Coord| merc_to_cartesian_coords((x.x, x.y), tl_br.0, tl_br.1, merc_scale)).collect();
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

pub struct DepthLayer {
    pub coordinates: Vec<(f64, f64, f64)>,
    pub font: sfml::SfBox<Font>
}

impl DepthLayer {
    pub fn new(coordinates: Vec<(f64, f64, f64)>, font: sfml::SfBox<Font>) -> DepthLayer {
        DepthLayer { coordinates, font }
    }
    pub fn project_coords(&mut self, tl_br: ((f64, f64), (f64, f64))) {
        let merc_scale = get_merc_scaling_size();
        let mut final_points: Vec<(f64, f64, f64)> = Vec::new();
        for point in &self.coordinates {
            let merc_point = mercator_transform((point.0, point.1), merc_scale);
            let (x, y) = merc_to_cartesian_coords(merc_point, tl_br.0, tl_br.1, merc_scale);
            final_points.push((x, y, point.2));
        }
        self.coordinates = final_points;
    }
}

impl Plotable for DepthLayer {
    fn render(&self, window: &mut RenderWindow, _: f32, _viewvec: (f32, f32), _: (u32, u32)) -> () {
        render_soundg(window, self, &self.font)
    }
    
}
pub fn get_soundg_layer(ds: & Dataset) -> Layer {
    let mut layers = get_layers(&ds, vec!["SOUNDG"]);
    let layer = layers.pop().unwrap();
    layer
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
                        //let merc_point = mercator_transform((point.0, point.1), merc_scale);
                        //let (x, y) = merc_to_cartesian_coords(merc_point, tl_br.0, tl_br.1, merc_scale);
                        final_points.push((point.0, point.1, point.2));

                    }
                }, 
                unsure => {
                    debug!("{geom_type_name} is {unsure}");
                }
            }
        }
    }
    DepthLayer { coordinates: final_points, font: get_default_font() } 
} 



pub fn get_default_font() -> sfml::SfBox<Font> {
    Font::from_file("./src/fonts/OpenSans-Regular.ttf").unwrap() 
}

pub fn get_extent_from_layers_in_ds(layer_names: &Vec<String>, dataset: &Dataset) -> ((f64, f64), (f64, f64)) {
    let mut min_extent = (f64::MAX, f64::MAX);
    let mut max_extent = (f64::MIN, f64::MIN);
    let layer_names_str = layer_names.iter().map(|x| x as &str).collect();
    let layers = get_layers(&dataset, layer_names_str);
    for  mut layer in layers {
        let layer_name = layer.name().clone();
        // debug!("Checking layer {}", layer_name);
        for feature in layer.features()
        {
            let geomentry = match feature.geometry() {
                Some(geo) => geo,
                None => {
                    debug!("[get_extent_from_layers] {} has no geometry!", layer_name);
                    continue;
                }
            };
            let envl = geomentry.envelope();
            let br_corner = (envl.MinX, envl.MinY);
            let tl_corner = (envl.MaxX, envl.MaxY);
            // info!("BR Corner: {:?}, TL Corner: {:?}", br_corner, tl_corner);
            if tl_corner.0 > max_extent.0 {
                max_extent.0 = tl_corner.0;
            }
            if tl_corner.1 > max_extent.1 {
                max_extent.1 = tl_corner.1;
            }
            if br_corner.0 < min_extent.0 {
                min_extent.0 = br_corner.0;
            }
            if br_corner.1 < min_extent.1 {
                min_extent.1 = br_corner.1;
            }


        }

    }
    //info!("Max extent: {:?}, Min extent: {:?}", max_extent, min_extent);
    (min_extent, max_extent)

}

