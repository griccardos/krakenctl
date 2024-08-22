use crate::{imagetools, input::Input, settings::Settings};
use image::EncodableLayout;

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
    pub debugging: bool,
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
impl Drop for Manager {
    fn drop(&mut self) {
        let context = libusb::Context::new().unwrap();
        let mut device_handle = context.open_device_with_vid_pid(0x1e71, 0x3008).expect("Could not open kraken for bulk");

        for &i in self.kernel_drivers.iter() {
            device_handle.attach_kernel_driver(i).unwrap();
            if self.debugging {
                println!("kernel attach:{i:?}");
            }
        }
        device_handle.reset().unwrap();
    }
}

#[allow(clippy::vec_init_then_push)]
impl Manager {
    pub fn new(debugging: bool, settings: Settings) -> Self {
        let mut vec = vec![];
        let context = libusb::Context::new().unwrap();

        let mut device_handle = context.open_device_with_vid_pid(0x1e71, 0x3008).expect("Could not open kraken");
        device_handle.reset().unwrap();

        if debugging {
            println!("config: {:?}", device_handle.active_configuration());
        }

        for i in [0, 1] {
            let has_k = device_handle.kernel_driver_active(i).unwrap_or_default();
            if has_k {
                device_handle.detach_kernel_driver(i).unwrap();
                vec.push(i);
                if debugging {
                    println!("kernel detach:{i:?}");
                }
            }
        }
        Manager {
            image_index: None,
            debugging,
            settings,
            kernel_drivers: vec,
        }
    }
    fn debug(&self, string: String) {
        if self.debugging {
            println!("{}", string);
        }
    }

    pub fn details(&self) -> String {
        String::new()
    }

    pub fn set_blank(&mut self) {
        self.write_to_interrupt(buff![0x38, 1 ;64]);
    }

    pub fn set_liquid(&mut self) {
        self.write_to_interrupt(buff![0x38,1,2;64]);
    }

    pub fn set_brightness(&mut self, brightness: u8) {
        if brightness > 100 {
            return;
        }

        self.write_to_interrupt(buff![0x30,2,1,brightness;64]);
    }

    pub fn loop_images(&mut self) {
        for i in 1..=16 {
            self.set_image_at_index(i);
            std::thread::sleep(Duration::from_millis(1500));
        }
    }

    fn set_image_at_index(&mut self, index: u8) {
        self.write_to_interrupt(buff![0x38,1,4,index;64]);
    }

    pub fn set_image_with_bytes(&mut self, img_bytes: &[u8], is_gif: bool) {
        if self.image_index.is_none() {
            //setup image writing
            // self.write_to_interrupt(buff![0x10,1;64]);
            // self.write_to_interrupt(buff![0x36,3;64]);
            // self.write_to_interrupt(buff![0x30,1;64]);

            //clear images
            for i in 0..16 {
                self.write_to_interrupt(buff![0x30,4,i;64]);
            }
            // self.write_to_interrupt(buff![0x20,3;64]);

            //self.write_to_interrupt(buff![0x30,2,1,0x42,0,0,1,2;64]);

            //self.write_to_interrupt(buff![0x72,2,0,0,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b;64]);
            //self.write_to_interrupt(buff![0x72,2,0,0,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45,0x45;64]);
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

        let index = (self.image_index.unwrap() % 16) as u8;
        self.image_index = Some(self.image_index.unwrap() + 1);

        //delete bucket
        self.write_to_interrupt(buff![0x32,2,index;64]);

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

        self.write_to_interrupt(setup_bytes);

        //start bulk write
        self.write_to_interrupt(buff![0x36,1,index; 64]);

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

        self.write_to_bulk(&header);
        self.write_to_bulk(img_bytes);

        //end bulk write
        self.write_to_interrupt(buff![0x36,2;64]);

        //show image at index
        self.set_image_at_index(index);
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
            status.liquid_temp, status.fan_speed, status.fan_rate, status.pump_speed, status.pump_rate, status.firmware.0, status.firmware.1, status.firmware.2
        );
    }
    fn query(&mut self) -> Status {
        //run query
        self.write_to_interrupt(buff![0x74,1;64]);
        let bytes = self.read_from_interrupt();

        self.write_to_interrupt(buff![0x10,1;64]);
        let bytes_firm = self.read_from_interrupt();

        // self.write_to_interrupt(buff![0x20,3;64]);
        // let bytes_light = self.read_from_interrupt();

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

    fn read_from_interrupt(&mut self) -> Vec<u8> {
        let mut context = libusb::Context::new().unwrap();
        if self.debugging {
            context.set_log_level(libusb::LogLevel::Debug);
        }
        let mut device_handle = context.open_device_with_vid_pid(0x1e71, 0x3008).expect("Could not open kraken for bulk");
        // let mut kernel_drivers = vec![];
        // for i in [0, 1] {
        //     let has_k = device_handle.kernel_driver_active(i).unwrap_or_default();
        //     if has_k {
        //         device_handle.detach_kernel_driver(i).unwrap();
        //         kernel_drivers.push(i);
        //         self.debug(format!("kernel detach:{i:?}"));
        //     }
        // }

        device_handle.set_active_configuration(1).unwrap();
        device_handle.claim_interface(1).expect("could not claim interface");

        let mut buf = [0u8; 64];
        let res = device_handle.read_interrupt(129, &mut buf, Duration::from_millis(500));
        device_handle.release_interface(1).expect("Could not release interface");
        self.debug(format!("read result; {:?} {:?}", res, &buf));

        // for &i in kernel_drivers.iter() {
        //     device_handle.attach_kernel_driver(i).unwrap();
        //     self.debug(format!("kernel attach:{i:?}"));
        // }
        buf.into_iter().collect::<Vec<u8>>()
    }

    fn write_to_interrupt(&mut self, bytes: Vec<u8>) {
        let mut context = libusb::Context::new().unwrap();
        if self.debugging {
            context.set_log_level(libusb::LogLevel::Debug);
        }
        let mut device_handle = context.open_device_with_vid_pid(0x1e71, 0x3008).expect("Could not open kraken for bulk");

        // let mut kernel_drivers = vec![];
        // for i in [0, 1] {
        //     let has_k = device_handle.kernel_driver_active(i).unwrap_or_default();
        //     if has_k {
        //         device_handle.detach_kernel_driver(i).unwrap();
        //         kernel_drivers.push(i);
        //         self.debug(format!("kernel detach:{i:?}"));
        //     }
        // }
        device_handle.set_active_configuration(1).unwrap();
        device_handle.claim_interface(1).unwrap();
        let result = device_handle.write_interrupt(1, &bytes, Duration::from_millis(200));
        if let Err(err) = result {
            println!("Error writing to interface {err}");
            return;
        }
        device_handle.release_interface(1).expect("Could not release interface");

        // for &i in kernel_drivers.iter() {
        //     device_handle.attach_kernel_driver(i).unwrap();
        //     self.debug(format!("kernel attach:{i:?}"));
        // }
    }

    fn write_to_bulk(&mut self, bytes: &[u8]) {
        let context = libusb::Context::new().unwrap();
        let mut device_handle = context.open_device_with_vid_pid(0x1e71, 0x3008).expect("Could not open kraken for bulk");
        device_handle.set_active_configuration(1).unwrap();
        device_handle.claim_interface(0).expect("Could not claim");
        device_handle.write_bulk(2, bytes, Duration::from_millis(100)).expect("Could not write to bulk");
        device_handle.release_interface(0).expect("Could not release interface");
    }

    pub fn set_values_from_input(&mut self, input: &str, time: bool) {
        let val = Input::new(input, time);
        let start = Instant::now();
        let im = imagetools::image_from_input(val, &self.settings);
        let elap1 = start.elapsed();
        self.set_image_with_bytes(&im, false);
        let elap2 = start.elapsed() - elap1;

        if self.debugging {
            println!("creating image took {}ms", elap1.as_millis());
            println!("setting image took {}ms", elap2.as_millis());
        }
    }
}
