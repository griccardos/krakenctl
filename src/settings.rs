use image::Rgba;
use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    time::SystemTime,
};

#[derive(Debug)]
pub struct Settings {
    pub time: Rgba<u8>,
    pub left_bar: Rgba<u8>,
    pub right_bar: Rgba<u8>,
    pub left_value: Rgba<u8>,
    pub right_value: Rgba<u8>,
    pub left_title: Rgba<u8>,
    pub right_title: Rgba<u8>,
    pub show_time: bool,
    pub loaded: SystemTime,
    pub path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            time: Rgba([255, 255, 255, 255]),
            left_bar: Rgba([120, 120, 255, 255]),
            right_bar: Rgba([120, 120, 255, 255]),
            left_value: Rgba([255, 255, 255, 255]),
            right_value: Rgba([255, 255, 255, 255]),
            left_title: Rgba([120, 120, 255, 255]),
            right_title: Rgba([120, 120, 255, 255]),
            show_time: false,
            loaded: SystemTime::UNIX_EPOCH,
            path: None,
        }
    }
}
impl Settings {
    pub fn get_file() -> Result<PathBuf, String> {
        let path = dirs::config_dir();
        if path.is_none() {
            return Err("No config dir".to_string()); //no config dir
        }
        let mut path = path.unwrap();
        path.push("krakenctl");
        path.push("config.ini");
        if !path.exists() {
            return Err(format!("No config file at {path:?}")); //no config file
        }
        Ok(path)
    }
    pub fn load() -> Result<Self, String> {
        let path = Settings::get_file()?;
        let file = File::open(&path);
        let Ok(file) = file else {
            return Err(format!("Could not open file {path:?}")); //no read permission
        };

        let mut reader = BufReader::new(&file);
        let mut buffer = String::new();

        let _ = reader.read_to_string(&mut buffer);
        let lines = buffer
            .split('\n')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|x| x.split('=').collect::<Vec<&str>>())
            .filter(|x| x.len() == 2)
            .map(|x| (x[0], x[1]))
            .collect::<Vec<(&str, &str)>>();

        let mut settings = Settings {
            loaded: SystemTime::now(),
            path: Some(path),
            ..Default::default()
        };

        for (left, right) in lines {
            match (left, right) {
                ("left_bar", right) => settings.left_bar = string_to_rgba(right),
                ("right_bar", right) => settings.right_bar = string_to_rgba(right),
                ("left_value", right) => settings.left_value = string_to_rgba(right),
                ("right_value", right) => settings.right_value = string_to_rgba(right),
                ("left_title", right) => settings.left_title = string_to_rgba(right),
                ("right_title", right) => settings.right_title = string_to_rgba(right),
                ("time", right) => settings.time = string_to_rgba(right),
                ("show_time", "true") => settings.show_time = true,

                _ => (),
            }
        }
        Ok(settings)
    }

    pub(crate) fn modified_time() -> Result<SystemTime, Box<dyn Error>> {
        let file = Settings::get_file()?;
        Ok(file.metadata()?.modified()?)
    }
}

fn string_to_rgba(string: &str) -> Rgba<u8> {
    let default_colour = Rgba([255, 0, 0, 255]);
    if string.len() < 7 || string.chars().next().unwrap_or_default() != '#' {
        return default_colour;
    }
    let mut vec = vec![];
    for item in string
        .chars()
        .skip(1)
        .take(6)
        .collect::<Vec<char>>()
        .chunks(2)
    {
        let val = u8::from_str_radix(format!("{}{}", item[0], item[1]).as_str(), 16);
        if let Ok(val) = val {
            vec.push(val);
        }
    }
    if vec.len() == 3 {
        Rgba([vec[0], vec[1], vec[2], 255])
    } else {
        default_colour
    }
}
