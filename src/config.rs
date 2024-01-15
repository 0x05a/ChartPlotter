use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use sfml::graphics::Color;
use toml::Table;

pub fn get_config() -> Table {
    let path = Path::new("config.toml");
    let mut file = File::open(&path).expect("Couldn't open config.toml");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Couldn't read config.toml");
    let config: Table = toml::from_str(&contents).expect("Couldn't parse config.toml");
    // dbg!(&config);
    config
}
pub fn get_resolution() -> (u32, u32) {
    let c = get_config();
    let res1 = c["resolution_1"].as_integer().unwrap() as u32;
    let res2 = c["resolution_2"].as_integer().unwrap() as u32;
    (res1, res2)
    //(1000, 1000)
}

pub fn get_layers() -> Vec<String> {
    let c = get_config();
    let layers = c["layers"].as_array().unwrap();
    let mut layers_vec = Vec::new();
    for layer in layers {
        layers_vec.push(layer.as_str().unwrap().to_string());
    }
    layers_vec

    // vec!["SEAARE"]
}
pub fn get_merc_scaling_size() -> (u32, u32) {
    let c = get_config();
    let merc_scaling_size_1 = c["merc_scaling_size_1"].as_integer().unwrap() as u32;
    let merc_scaling_size_2 = c["merc_scaling_size_2"].as_integer().unwrap() as u32;
    (merc_scaling_size_1, merc_scaling_size_2)
    //   (1_000, 1_000)
}
pub fn get_chart_path() -> String {
    let c = get_config();
    c["chart_path"].as_str().unwrap().to_string()
    //   "/home/zack/chart_plotter/rust_rewrite/chartplotter/src/charts/BYU_CHICO.000"
}

pub fn get_init_tl_br() -> ((f64, f64),(f64, f64)) {
    let c = get_config();
    let top_left_1 = c["top_left_1"].as_float().unwrap();
    let top_left_2 = c["top_left_2"].as_float().unwrap();
    let bottom_right_1 = c["bottom_right_1"].as_float().unwrap();
    let bottom_right_2 = c["bottom_right_2"].as_float().unwrap();
    ((top_left_1, top_left_2), (bottom_right_1, bottom_right_2))
//    ((30.375, -87.3 ), (30.45, -87.225))
}


pub fn get_color_for_layer(layer_name: &str) -> Color {
    let c = get_config();
    let layer_table = c[layer_name].as_table().unwrap();
    let v = layer_table.get("color").unwrap();
    let color = {
        match v {
            toml::Value::Array(a) => {
                let r = a[0].as_integer().unwrap() as u8;
                let g = a[1].as_integer().unwrap() as u8;
                let b = a[2].as_integer().unwrap() as u8;
                if a.len() == 3 {
                    return Color::rgb(r, g, b)
                }
                else if a.len() == 4  {
                    let a = a[3].as_integer().unwrap() as u8;
                    Color::rgba(r, g, b, a)
                }
                else {
                    println!("Wrong number of color values: {} expected 3 or 4", a.len());
                    println!("Defaulting to RED");
                    Color::RED
                }
            },
            toml::Value::String(s) => {
                match s.to_uppercase().as_str() {
                    "RED" => Color::RED,
                    "GREEN" => Color::GREEN,
                    "BLUE" => Color::BLUE,
                    "YELLOW" => Color::YELLOW,
                    "MAGENTA" => Color::MAGENTA,
                    "CYAN" => Color::CYAN,
                    "TRANSPARENT" => Color::TRANSPARENT,
                    _ => {
                        println!("Unknown color: {}", s);
                        println!("Defaulting to RED");
                        Color::RED
                    
                    },
                }
            },
            _ => {
                println!("Unknown color: {:?}", v);
                println!("Defaulting to RED");
                Color::RED
            },
        }
    };
    color
    // (255, 255, 255)
}