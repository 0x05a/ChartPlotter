use std::f64::consts::PI;

//use log::{debug, info};

pub fn mercator_transform(coord: (f64, f64), scale: (u32, u32)) -> (f64, f64)
{
    let lon = coord.0;
    let lat = coord.1;
    //debug!("Transforming lon: {}, lat: {}", lon, lat);
    let (width, height) = scale;

    let transformed_lon = (width as f64  * (lon + 180.0) / 360.0) % (width as f64 + width as f64 / 2.0);
    let lat_rad = lat * PI / 180.0;
    let merc_n: f64 = ((PI / 4.0) + (lat_rad / 2.0)).tan().ln();
    let transformed_lat = (height as f64 / 2.0) - (width as f64 * merc_n / (2.0 * PI));
    (transformed_lon, transformed_lat)
}

pub fn merc_to_cartesian_coords(coord: (f64, f64), tl_corner: (f64, f64), br_corner: (f64, f64), scale: (u32, u32)) -> (f64, f64) {
    let (width, height) = scale;
    let x = coord.0;
    let y = coord.1;
    //debug!("coord: {:?}", coord);
    //debug!("tl_corner: {:?}", tl_corner);
    //debug!("br_corner: {:?}", br_corner);
    let transformed_br = mercator_transform(br_corner, scale);
    let transformed_tl = mercator_transform(tl_corner, scale);
    //debug!("Transformed BR: {:?}, Transformed TL: {:?}", transformed_br, transformed_tl);
    let max_lat = transformed_tl.1;
    let min_lat = transformed_br.1;
    let max_lon = transformed_tl.0;
    let min_lon = transformed_br.0;
    //info!("Max Lat: {}, Min Lat: {}, Max Lon: {}, Min Lon: {}", max_lat, min_lat, max_lon, min_lon);

    let merc_lat_delta = max_lat - min_lat;
    let merc_lon_delta = max_lon - min_lon;
    let merc_lat_ratio = (y - min_lat) / merc_lat_delta;
    let merc_lon_ratio = (x - min_lon) / merc_lon_delta;
    //debug!("Coordinate after scaling by mercator transform: {:?}", (merc_lon_ratio, merc_lat_ratio));
    //debug!("Coordinate after scaling by width and height: {:?}", (merc_lon_ratio * width as f64, merc_lat_ratio * height as f64));
    // our x in this case is latitude and our y is longitude
    (merc_lon_ratio * width as f64, height as f64 - merc_lat_ratio * height as f64)
}
