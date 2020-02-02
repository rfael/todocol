#[macro_use]
extern crate clap;

mod app_core;

use clap::{App, Arg};
use env_logger::Builder;
use log::LevelFilter;
use log::{debug, error, info, trace};

macro_rules! simple_error_result {
    ($fmt:literal $(, $x:expr )*) => {{
        Err(Box::new(simple_error::SimpleError::new(format!($fmt, $($x),*))))
    }};
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("config json file")
                .default_value("$HOME/.config/todocol/settings.json")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("filename")
                .short("n")
                .long("filename")
                .help("output files name")
                .default_value("TODO")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("format")
                .short("f")
                .long("format")
                .help("output files format")
                .possible_value("txt")
                .possible_value("markdown")
                .default_value("txt")
                .takes_value(true),
        )
        .subcommand(
            App::new("project")
                .about("Collects comments in one specified project directory")
                .arg(
                    Arg::with_name("path")
                        .short("p")
                        .help("Path to directory")
                        .default_value("$PWD")
                        .index(1),
                ),
        )
        .subcommand(
            App::new("workspace")
                .about("Collects comments from all directories in one workspace")
                .arg(
                    Arg::with_name("path")
                        .short("p")
                        .help("Path to directory")
                        .default_value("$PWD")
                        .index(1),
                ),
        )
        .get_matches();

    // verbosity
    let level_filter = match matches.occurrences_of("verbose") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    Builder::new().filter_level(level_filter).init();
    debug!("Debug logs enabled");

    // app settings
    let mut settings = config::Config::default();
    settings.set_default("prefix", vec!["TODO"]).unwrap();
    settings.set_default("comment_symbol", vec!["//"]).unwrap();
    settings.set_default("outfile.name", "TODO").unwrap();
    settings.set_default("outfile.format", "txt").unwrap();

    let config_file = matches.value_of("config").unwrap_or("$HOME/.config/todocol/settings.json");
    let config_file = app_core::swap_env(config_file);
    info!("Config file: {}", config_file);
    match settings.merge(config::File::with_name(&config_file)) {
        Ok(_s) => info!("Config file: {}", config_file),
        Err(e) => error!("{}", e),
    }

    if matches.occurrences_of("format") == 1 {
        if let Some(format) = matches.value_of("format") {
            settings.set("outfile.format", format).unwrap();
        }
    }
    if matches.occurrences_of("filename") == 1 {
        if let Some(name) = matches.value_of("filename") {
            settings.set("outfile.name", name).unwrap();
        }
    }

    trace!("App config:\n{:#?}", settings);

    match matches.subcommand_name() {
        Some("project") => {
            info!("Collecting todo for project");
            if let Some(project_dir) = matches.subcommand_matches("project").and_then(|m| m.value_of("path")) {
                app_core::run_app_project(&settings, project_dir)
            } else {
                simple_error_result!("Invalid argument")
            }
        }
        Some("workspace") => {
            info!("Collecting todo for workspace");
            if let Some(workspace_dir) = matches.subcommand_matches("workspace").and_then(|m| m.value_of("path")) {
                app_core::run_app_workspace(&settings, workspace_dir)
            } else {
                simple_error_result!("Invalid argument")
            }
        }
        Some("workspaces") => {
            info!("Collecting todo in all workspaces");
            app_core::run_app_workspaces(&settings)
        }
        _ => {
            eprintln!("{}", matches.usage());
            Ok(())
        }
    }
}
