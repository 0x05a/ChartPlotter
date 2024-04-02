use std::f64::consts::PI;

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
