#![allow(dead_code)]
#![allow(non_snake_case)]

mod imagetools;
mod input;
mod managerrusb;
mod settings;

use clap::Parser;
use managerrusb::{DebugLevel, Manager};
use settings::Settings;
#[cfg(target_os = "linux")]
use signal_hook::consts::{SIGHUP, SIGTSTP};
use signal_hook::consts::{SIGINT, SIGTERM};
use std::{
    sync::{atomic::AtomicUsize, Arc},
    thread::sleep,
    time::{Duration, Instant},
};
use systemstat::Platform;

#[derive(Parser)]
#[command(about = "Change display of Kraken devices. Use at your own risk!")]
struct Cli {
    #[arg(short, long, help = "Displays liquid screen")]
    liquid: bool,
    #[arg(short, long, help = "Displays blank screen")]
    blank: bool,

    #[arg(long, short = 'k', help = "Set brightness (0-100)")]
    brightness: Option<u8>,

    #[arg(
        long,
        help = "Run script, and get output as values. Expects same output as --values",
        hide = true
    )]
    script: Option<String>,

    #[arg(long, help = "Load image", hide = true)]
    image: Option<String>,

    #[arg(long, help = "Load animated gif", hide = true)]
    gif: Option<String>,

    #[arg(long, help = "Show time")]
    time: bool,

    #[arg(
        short,
        long,
        help = "Debug mode",
        num_args= 0..=1,//required for optional argument
        default_missing_value = "3", //for when no value added
        default_value = "0" //for when not added
    )]
    debug: String,

    #[arg(long, help = "Read device status")]
    status: bool,

    #[arg(short, long, help = "Displays CPU temperature")]
    cpu: bool,

    #[arg(
        short,
        long,
        help = "Displays 1 or 2 values from string (optionally comma separated for multiple values). one can include units also.\nDue to limited space keep each value to 3 or 4 characters\nOptional ';' with titles (remember to wrap in quotes)\nExamples of valid values:\n45°\n\"45;CPU\"\n'45°,32°;CPU,GPU'"
    )]
    values: Option<String>,

    #[arg(
        short,
        long,
        help = "Repeat every X seconds only applicable to: script,cpu,gpu,values"
    )]
    repeat: Option<u64>,
}

fn main() {
    let start = Instant::now();

    let clapp = Cli::parse();

    let debug_level = clapp.debug.into();

    let settings = match Settings::load() {
        Ok(s) => s,
        Err(e) => {
            if debug_level >= DebugLevel::Info {
                println!("Could not load settings, using default. {e}");
            }
            Settings::default()
        }
    };

    let time = clapp.time;
    if debug_level >= DebugLevel::Info {
        println!("{settings:?}");
    }

    let mut manager = match Manager::new(debug_level, settings) {
        Ok(m) => m,
        Err(e) => {
            println!("Could not create manager, {e}");
            std::process::exit(1);
        }
    };

    if clapp.liquid {
        manager.set_liquid();
    } else if clapp.blank {
        manager.set_blank();
    } else if clapp.status {
        manager.print_status();
    } else if let Some(br) = clapp.brightness {
        if br <= 100 {
            manager.set_brightness(br);
        } else {
            println!("Brightness needs to be between 0 and 100");
        }
    } else if clapp.cpu {
        maybe_repeat(
            move || {
                let ss = systemstat::System::new();
                if let Ok(temp) = ss.cpu_temp() {
                    println!("temp is {temp}");
                    manager.set_values_from_input(&format!("{temp}°"), time);
                } else {
                    println!("Getting CPU temp is not supported on this platform");
                }
            },
            clapp.repeat.clone(),
        );
    } else if let Some(input) = clapp.values {
        manager.set_values_from_input(&input, time)
    } else if let Some(path) = clapp.script {
        if debug_level >= DebugLevel::Info {
            println!("running script '{path}'");
        }
        maybe_repeat(
            move || {
                let output = std::process::Command::new(&path)
                    .output()
                    .unwrap_or_else(|err| panic!("Could not run script '{path}': {err}"));
                let (stdo, stde, status) = (
                    String::from_utf8(output.stdout)
                        .unwrap_or_default()
                        .replace("\n", "")
                        .replace("\r", ""),
                    String::from_utf8(output.stderr).unwrap_or_default(),
                    output.status,
                );
                if debug_level >= DebugLevel::Info {
                    println!("out:'{stdo}' err:'{stde}' status:{status:?}")
                }

                manager.set_values_from_input(&stdo, time);
                manager.reload_settings();
            },
            clapp.repeat.clone(),
        );
    } else if let Some(path) = clapp.image {
        manager.set_image(&path)
    } else if let Some(path) = clapp.gif {
        manager.set_gif(&path)
    }

    if debug_level >= DebugLevel::Info {
        println!("Ran in {}ms", start.elapsed().as_millis());
    }
}

fn maybe_repeat<F: FnMut()>(mut func: F, rep: Option<u64>) {
    let term: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    signal_hook::flag::register_usize(SIGTERM, Arc::clone(&term), SIGTERM as usize).unwrap();
    signal_hook::flag::register_usize(SIGINT, Arc::clone(&term), SIGINT as usize).unwrap();
    #[cfg(target_os = "linux")]
    {
        signal_hook::flag::register_usize(SIGTSTP, Arc::clone(&term), SIGTSTP as usize).unwrap();
        signal_hook::flag::register_usize(SIGHUP, Arc::clone(&term), SIGHUP as usize).unwrap();
    }
    loop {
        func(); //run the function

        if let Some(repeat) = rep {
            sleep(Duration::from_secs(repeat));
        } else {
            break;
        }

        let sig = term.load(std::sync::atomic::Ordering::Relaxed);
        match sig {
            0 => (),
            signal => {
                eprintln!("Got signal to exit with code {signal}");
                if let Ok(mut man) = Manager::new(DebugLevel::None, Settings::default()) {
                    man.set_liquid();
                }
                break;
            }
        }
    }
}
