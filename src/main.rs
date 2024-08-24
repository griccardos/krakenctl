#![allow(dead_code)]
#![allow(non_snake_case)]

mod imagetools;
mod input;
mod managerrusb;
mod settings;

use clap::Parser;
use managerrusb::Manager;
use run_script::ScriptOptions;
use settings::Settings;
use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::sleep,
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(about = "Change display of Kraken devices. Use at your own risk!")]
struct Cli {
    #[arg(
        short,
        long,
        help = "Displays values from string (optionally comma separated for multiple values) one can include units also.\nDue to limited space keep each value to 3 or 4 characters\nOptional ';' with titles (remember to wrap in quotes)\nExamples of valid values:\n45°\n\"45;CPU\"\n'45°,32°;CPU,GPU'"
    )]
    values: Option<String>,
    #[arg(short, long, help = "Displays liquid screen")]
    liquid: bool,
    #[arg(short, long, help = "Displays blank screen")]
    blank: bool,

    #[arg(long, help = "Set brightness (0-100)")]
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

    #[arg(short, long, help = "Debug mode")]
    debug: bool,

    #[arg(long, help = "Read device status")]
    status: bool,
}

fn main() {
    let start = Instant::now();

    let clapp = Cli::parse();

    let settings = Settings::load();

    let debugging = clapp.debug;
    let time = clapp.time;
    if debugging {
        println!("{settings:?}");
    }

    let mut manager = Manager::new(debugging, settings);

    //setup forced exit scenarios
    let must_exit = Arc::new(AtomicBool::new(false));

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
    } else if let Some(input) = clapp.values {
        manager.set_values_from_input(&input, time);
    } else if let Some(path) = clapp.script {
        loop {
            //exit cleanly, and relase usb
            let (_code, output, _error) = run_script::run(&path, &vec![], &ScriptOptions::new())
                .unwrap_or_else(|_| panic!("Could not run script {path}"));
            manager.set_values_from_input(&output, time);

            if must_exit.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            sleep(Duration::from_secs(1));
        }
    } else if let Some(path) = clapp.image {
        manager.set_image(&path)
    } else if let Some(path) = clapp.gif {
        manager.set_gif(&path)
    }

    if debugging {
        println!("Ran in {}ms", start.elapsed().as_millis());
    }
}
