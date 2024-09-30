use crate::{imagetools, input::Input, settings::Settings};
use chrono::{DateTime, Utc};
use image::EncodableLayout;
use rusb::{Context, DeviceHandle, LogLevel, UsbContext};

use std::{
    fs::File,
    io::{BufReader, Read},
    time::{Duration, Instant},
    u8,
};

pub struct Endpoint {
    config: u8,
    interface: u8,
    settings: u8,
    address: u8,
}

#[derive(Default)]
pub struct Status {
    pub liquid_temp: f32,
    pub pump_speed: usize,
    pub fan_speed: usize,
    pub pump_rate: u8,
    pub fan_rate: u8,
    pub firmware: (u8, u8, u8),
}

pub struct Manager {
    settings: Settings,
    image_index: Option<usize>,
    pub debug_level: DebugLevel,
    kernel_drivers: Vec<u8>,
}

macro_rules! buff {
    ($s:expr) => { //empty, just 0s
        vec![u8;$s]
    };
    ($($x:expr), +;$s:expr) => {{ //some data before
        let mut v:Vec<u8> = Vec::new();

        $(
        v.push($x);
        )+
        while v.len()<$s{
            v.push(0);
        }
        v

    }};
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum DebugLevel {
    None,
    Error,
    Warning,
    Info,
    Debug,
}
impl From<DebugLevel> for LogLevel {
    fn from(value: DebugLevel) -> Self {
        match value {
            DebugLevel::None => LogLevel::None,
            DebugLevel::Info => LogLevel::Info,
            DebugLevel::Debug => LogLevel::Debug,
            DebugLevel::Error => LogLevel::Error,
            DebugLevel::Warning => LogLevel::Warning,
        }
    }
}
impl From<String> for DebugLevel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "0" => DebugLevel::None,
            "1" => DebugLevel::Error,
            "2" => DebugLevel::Warning,
            "3" => DebugLevel::Info,
            "4" => DebugLevel::Debug,
            _ => DebugLevel::Debug,
        }
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        let context = rusb::Context::new().unwrap();
        if let Some(device_handle) = context.open_device_with_vid_pid(0x1e71, 0x3008) {
            for &i in self.kernel_drivers.iter() {
                if device_handle.attach_kernel_driver(i).is_ok() {
                    self.debug(format!("kernel attach:{i:?}"), DebugLevel::Debug);
                } else {
                    self.debug(format!("could not attach kernel driver"), DebugLevel::Debug);
                }
            }
            if device_handle.reset().is_err() {
                eprintln!("Could not reset device handle");
            }
        }
    }
}

#[allow(clippy::vec_init_then_push)]
impl Manager {
    pub fn new(debug_level: DebugLevel, settings: Settings) -> Self {
        let mut vec = vec![];
        let context = rusb::Context::new().unwrap();

        let device_handle = context
            .open_device_with_vid_pid(0x1e71, 0x3008)
            .expect("Could not open kraken");
        device_handle.reset().unwrap();

        if debug_level >= DebugLevel::Info {
            println!("config: {:?}", device_handle.active_configuration());
        }

        for i in [0, 1] {
            let has_k = device_handle.kernel_driver_active(i).unwrap_or_default();
            if has_k {
                device_handle.detach_kernel_driver(i).unwrap();
                vec.push(i);
                if debug_level >= DebugLevel::Debug {
                    println!("kernel detach:{i:?}");
                }
            }
        }
        Manager {
            image_index: None,
            debug_level,
            settings,
            kernel_drivers: vec,
        }
    }
    fn debug(&self, string: String, level: DebugLevel) {
        if self.debug_level >= level {
            println!("{}", string);
        }
    }

    pub fn details(&self) -> String {
        String::new()
    }

    pub fn set_blank(&mut self) {
        if let Some(dev) = self.get_handle() {
            self.write_to_interrupt(&dev, buff![0x38, 1 ;64]);
        }
    }

    pub fn set_liquid(&mut self) {
        if let Some(device_handle) = self.get_handle() {
            self.write_to_interrupt(&device_handle, buff![0x38,1,2;64]);
        }
    }

    pub fn set_brightness(&mut self, brightness: u8) {
        if brightness > 100 {
            return;
        }
        if let Some(device_handle) = self.get_handle() {
            self.write_to_interrupt(&device_handle, buff![0x30,2,1,brightness;64]);
        }
    }

    pub fn loop_images(&mut self) {
        if let Some(device_handle) = self.get_handle() {
            for i in 1..=16 {
                self.set_image_at_index(&device_handle, i);
                std::thread::sleep(Duration::from_millis(1500));
            }
        }
    }

    fn set_image_at_index(&mut self, device_handle: &DeviceHandle<Context>, index: u8) {
        self.write_to_interrupt(&device_handle, buff![0x38,1,4,index;64]);
    }

    pub fn set_image_with_bytes(&mut self, img_bytes: &[u8], is_gif: bool) {
        if let Some(device_handle) = self.get_handle() {
            if self.image_index.is_none() {
                //clear images
                for i in 0..16 {
                    self.write_to_interrupt(&device_handle, buff![0x30,4,i;64]);
                }

                let rand_index: usize = rand::random::<usize>() % 16;
                self.image_index = Some(rand_index);
            }

            let mut mem_index = [[0u8; 2]; 16];
            mem_index[0] = [0x00, 0x00];
            mem_index[1] = [0x90, 0x01];
            mem_index[2] = [0x20, 0x03];
            mem_index[3] = [0xb0, 0x04];
            mem_index[4] = [0x40, 0x06];
            mem_index[5] = [0xd0, 0x07];
            mem_index[6] = [0x60, 0x09];
            mem_index[7] = [0xf0, 0x0a];
            mem_index[8] = [0x80, 0x0c];
            mem_index[9] = [0x10, 0x0e];
            mem_index[10] = [0xa0, 0x0f];
            mem_index[11] = [0x30, 0x11];
            mem_index[12] = [0xc0, 0x12];
            mem_index[13] = [0x50, 0x14];
            mem_index[14] = [0xe0, 0x15];
            mem_index[15] = [0x70, 0x17];

            //delete bucket
            for i in 0..2 {
                //let random = rand::random::<usize>() % 16;
                self.image_index = Some(i % 16);
                let index = (self.image_index.unwrap() % 16) as u8;

                self.image_index = Some(self.image_index.unwrap() + 1);

                self.write_to_interrupt(&device_handle, buff![0x32,2,index;64]);

                let setup_bytes = buff![
            0x32,
            1,
            index,
            index+1,
            mem_index[index as usize][0],
            mem_index[index as usize][1],
            0x90,
            1,
            1
            ;64];

                self.write_to_interrupt(&device_handle, setup_bytes);
                let delay = 100;
                //start bulk write
                self.write_to_interrupt(&device_handle, buff![0x36,1,index; 64]);
                std::thread::sleep(Duration::from_millis(delay));
                //BULK
                let mut header = buff![
        0x12,
        0xfa,
        0x01,
        0xe8,
        0xab,
        0xcd,
        0xef,
        0x98,
        0x76,
        0x54,
        0x32,
        0x10,
         0x2,
         0x0,
         0x0,
         0x0,
         0x0,
        0x40,
         0x6
         ;512];
                if is_gif {
                    header[12] = 1;
                }

                self.write_to_bulk(&device_handle, &header);
                std::thread::sleep(Duration::from_millis(delay));
                self.write_to_bulk(&device_handle, img_bytes);

                //end bulk write
                self.write_to_interrupt(&device_handle, buff![0x36,2;64]);
                //wait for image to finish sending

                //show image at index
                self.set_image_at_index(&device_handle, index);
            }
        }
    }

    pub fn set_image(&mut self, path: &str) {
        let img4 = imagetools::convert_image_from_path(path);
        let img5 = img4.to_rgba8();
        let img_bytes = img5.as_bytes();
        self.set_image_with_bytes(img_bytes, false);
    }

    pub fn set_gif(&mut self, path: &str) {
        //  let img4=imagetools::convert_image_from_path(path);
        //let img_bytes = img4.as_bytes();

        let f = File::open(path).unwrap();
        let mut img_bytes = vec![];
        let mut reader = BufReader::new(f);
        reader.read_to_end(&mut img_bytes).unwrap();

        self.set_image_with_bytes(&img_bytes, true);
    }

    pub fn print_status(&mut self) {
        let status = self.query();
        println!(
            "Liquid {}°C
Fan Speed {} rpm
Fan Rate {}%
Pump Speed {} rpm
Pump Rate {}%
Firmware {}.{}.{}",
            status.liquid_temp,
            status.fan_speed,
            status.fan_rate,
            status.pump_speed,
            status.pump_rate,
            status.firmware.0,
            status.firmware.1,
            status.firmware.2
        );
    }
    fn query(&mut self) -> Status {
        let bytes = self.write_and_read_interface(&buff![0x74, 1; 64]);
        let bytes_firm = self.write_and_read_interface(&buff![0x10, 1; 64]);

        if bytes.len() != 64 || bytes_firm.len() != 64 {
            return Status::default();
        }

        Status {
            liquid_temp: bytes[15] as f32 + bytes[16] as f32 / 10.,
            pump_speed: (bytes[18] as usize) << 8 | bytes[17] as usize,
            pump_rate: bytes[19],
            fan_speed: (bytes[24] as usize) << 8 | bytes[23] as usize,
            fan_rate: bytes[25],
            firmware: (bytes_firm[17], bytes_firm[18], bytes_firm[19]),
        }
    }

    fn get_handle(&mut self) -> Option<DeviceHandle<Context>> {
        let mut context = rusb::Context::new().unwrap();
        context.set_log_level(self.debug_level.into());
        let device_handle = context.open_device_with_vid_pid(0x1e71, 0x3008)?;

        device_handle.set_active_configuration(1).unwrap();
        Some(device_handle)
    }

    fn write_to_interrupt(&mut self, device_handle: &DeviceHandle<Context>, bytes: Vec<u8>) {
        if device_handle.claim_interface(1).is_err() {
            eprintln!("Could not claim interrupt interface");
            return;
        }
        let result = device_handle.write_interrupt(1, &bytes, Duration::from_millis(200));
        if let Err(err) = result {
            eprintln!("Error writing to interface {err}");
            return;
        }
        if device_handle.release_interface(1).is_err() {
            eprint!("Could not release interface");
        }
    }

    fn write_to_bulk(&mut self, device_handle: &DeviceHandle<Context>, bytes: &[u8]) {
        if let Err(err) = device_handle.claim_interface(0) {
            eprintln!("Could not claim bulk interface {}", err);
        } else {
            if device_handle
                .write_bulk(2, bytes, Duration::from_millis(200))
                .is_err()
            {
                eprintln!("Could not write to bulk");
            }
            if device_handle.release_interface(0).is_err() {
                eprintln!("Could not release interface");
            }
        }
    }

    pub fn set_values_from_input(&mut self, input: &str, time: bool) {
        self.debug(format!("creating image from '{input}'"), DebugLevel::Info);
        //we strip any newlines from the input and trim ends
        let input = input.replace("\n", "").replace("\r", "");
        let input = input.trim();

        let val = Input::new(input, time);
        let start = Instant::now();
        let im = imagetools::image_from_input(val, &self.settings);
        let elap1 = start.elapsed();
        self.set_image_with_bytes(&im, false);
        let elap2 = start.elapsed() - elap1;

        self.debug(
            format!("creating image took {}ms", elap1.as_millis()),
            DebugLevel::Info,
        );
        self.debug(
            format!("setting image took {}ms", elap2.as_millis()),
            DebugLevel::Info,
        );
    }
    fn write_and_read_interface(&mut self, input: &[u8]) -> Vec<u8> {
        let mut context = rusb::Context::new().unwrap();
        context.set_log_level(self.debug_level.into());
        let device_handle = context
            .open_device_with_vid_pid(0x1e71, 0x3008)
            .expect("Could not open kraken for bulk");

        device_handle.set_active_configuration(1).unwrap();
        device_handle
            .claim_interface(1)
            .expect("could not claim interface");

        let res = device_handle.write_interrupt(1, input, Duration::from_millis(200));
        if let Err(e) = res {
            println!("error writing to kraken {:?}", e);
        }
        std::thread::sleep(Duration::from_millis(200));
        let mut buf = [0u8; 64];
        let res = device_handle.read_interrupt(129, &mut buf, Duration::from_millis(500));
        if let Err(e) = res {
            println!("error reading from kraken {:?}", e);
        }
        let bytes = buf.into_iter().collect::<Vec<u8>>();

        device_handle
            .release_interface(1)
            .expect("Could not release interface");
        device_handle.reset().expect("Could not reset device");

        bytes
    }

    pub(crate) fn reload_settings(&mut self) {
        if let Some(time) = Settings::modified_time() {
            if time > self.settings.loaded {
                if self.debug_level >= DebugLevel::Info {
                    let ftime: DateTime<Utc> = time.into();
                    let stimea: DateTime<Utc> = self.settings.loaded.into();
                    println!(
                        "Reloading settings as file time {}> loaded time{}",
                        ftime.format("%Y-%m-%d %H:%M:%S"),
                        stimea.format("%Y-%m-%d %H:%M:%S")
                    );
                }
                self.settings = Settings::load().unwrap();
            }
        }
    }
}
