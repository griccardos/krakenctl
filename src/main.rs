#![allow(dead_code)]
#![allow(non_snake_case)]

//mod api;
mod imagetools;
mod input;
//mod managerlibusb;
//mod managerusbxpress;
mod managerrusb;
mod settings;

use clap::{Arg, Command};
use managerrusb::Manager;
//use managerusbxpress::Manager;
use run_script::ScriptOptions;
use settings::Settings;
use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::sleep,
    time::{Duration, Instant},
};

fn main() {
    let start = Instant::now();

    let examples = "45°\n\"45;CPU\"\n'45°,32°;CPU,GPU'";
    let clapp = Command::new("krakenctl")
        .version("v0.3")
        .about("Change display of Kraken devices. Use at your own risk!")
        .arg_required_else_help(true)
        .arg(
            Arg::new("values")
                .short('v')
                .long("values")
                .help(format!("Displays values from string (optionally comma separated for multiple values) one can include units also.\nDue to limited space keep each value to 3 or 4 characters\nOptional ';' with titles (remember to wrap in quotes)\nExamples of valid values:\n{examples}").as_str())
                .takes_value(true),
        )
        .arg(
            Arg::new("liquid")
                .short('l')
                .long("liquid")
                .help("Displays liquid screen"),
        )
        .arg(
            Arg::new("blank")
                .short('b')
                .long("blank")
                .help("Displays blank screen"),
        )
        .arg(
            Arg::new("brightness")
                .long("brightness")
                .help("Set brightness (0-100)")
                .takes_value(true),
        )
        .arg(
            Arg::new("script")
                .long("script")
                .help("Run script, and get output as values. Expects same output as --values")
                .takes_value(true)
                .hide(true)
        )
        .arg(
            Arg::new("image")
                .long("image")
                .help("set image")
                .takes_value(true)
                .hide(true)
                )
        .arg(
            Arg::new("gif")
                .long("gif")
                .help("set animated gif")
                .takes_value(true)
                .hide(true)
                )
        .arg(
            Arg::new("time")
                .long("time")
                .help("show time")
                )
        .arg(
            Arg::new("debug")
            .short('d')
            .hide(true)
            )
            .arg(
                Arg::new("status")
                .long("status")
                .help("Read device status")
                .hide(false)
            )

        .get_matches();

    let settings = Settings::load();

    let debugging = clapp.is_present("debug");
    let time = clapp.is_present("time");

    let mut manager = Manager::new(debugging, settings);

    //setup forced exit scenarios
    let must_exit = Arc::new(AtomicBool::new(false));
    // let mut must_exit_clone = must_exit.clone();
    // thread::spawn(move || {
    //     let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
    //     for sig in signals.forever() {
    //         if debugging {
    //             println!("Received signal {:?}", sig);
    //         }
    //         must_exit_clone.borrow_mut().store(true, std::sync::atomic::Ordering::Relaxed);
    //     }
    // });
    /////

    if clapp.is_present("liquid") {
        manager.set_liquid();
    } else if clapp.is_present("blank") {
        manager.set_blank();
    } else if clapp.is_present("status") {
        manager.print_status();
    } else if clapp.is_present("brightness") {
        if let Some(br) = clapp.value_of("brightness") {
            let res = br.parse::<u8>();
            println!("'{br}':{res:?}");
            if let Ok(br) = br.parse::<u8>() {
                manager.set_brightness(br);
            } else {
                println!("Brightness needs to be between 0 and 100");
            }
        } else {
            println!("Brightness needs to be provided");
        }
    } else if clapp.is_present("values") {
        if let Some(input) = clapp.value_of("values") {
            manager.set_values_from_input(input, time);
        } else {
            println!("pass parameters such as {examples}");
        }
    } else if clapp.is_present("script") {
        if let Some(path) = clapp.value_of("script") {
            loop {
                //exit cleanly, and relase usb

                let (_code, output, _error) = run_script::run(path, &vec![], &ScriptOptions::new())
                    .unwrap_or_else(|_| panic!("Could not run script {path}"));
                manager.set_values_from_input(&output, time);

                if must_exit.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                sleep(Duration::from_secs(1));
            }
        } else {
            println!("pass parameters such as {examples}");
        }
    } else if clapp.is_present("image") {
        if let Some(path) = clapp.value_of("image") {
            manager.set_image(path)
        }
    } else if clapp.is_present("gif") {
        if let Some(path) = clapp.value_of("gif") {
            manager.set_gif(path)
        }
    }

    if debugging {
        println!("Ran in {}ms", start.elapsed().as_millis());
    }
}
