mod brain;
mod config;
mod log;

use crate::config::{Action, Data, Named, Template};
use crate::log::init_log;
use ::log::{info, LevelFilter};
use clap::{arg, command, ArgAction, Command};
use core::panic;
use std::error::Error;
use std::fs::{self, File};
use std::io::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let cli_result = init_cli();
    let log_filter = match cli_result
        .get_one::<u8>("debug")
        .expect("Count's are defaulted")
    {
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Error,
    };
    init_log(log_filter).expect("Logging must be successfully initialized");

    // Initialize config directory and prepare config file
    let config_file_path = Template::config_dir().join("config.toml");
    info!("{:?}", config_file_path);
    fs::create_dir_all(Template::config_dir()).expect("Must have access to create config folder");
    let mut file = if config_file_path.exists() {
        File::open(config_file_path).unwrap()
    } else {
        //TODO: fill with default config
        File::create(config_file_path).unwrap()
    };

    // Read config file
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let data: Data = toml::from_str(&contents)?;

    info!("Config: {:?}", data.config);
    info!("Templates: {:?}", data.templates);

    let action: Action;
    let mut name: Option<String> = None;
    let mut template: Option<String> = None;

    // Parse out name and template
    // Then set the action to the corresponding type
    parse_template_arg(&cli_result, &mut template)?;
    parse_name_arg(&data, &cli_result, &mut template, &mut name);
    if let Some(matches) = cli_result.subcommand_matches("new") {
        parse_template_arg(matches, &mut template)?;
        parse_name_arg(&data, matches, &mut template, &mut name);
        action = Action::New(Named::from_template_and_name(template, name));
    } else if let Some(matches) = cli_result.subcommand_matches("add") {
        parse_name_arg(&data, matches, &mut template, &mut name);
        action = Action::Add(Named::from_template_and_name(template, name));
    } else if cli_result.subcommand_matches("list").is_some() {
        action = Action::List;
    } else {
        action = Action::Default(Named::from_template_and_name(template, name));
    }

    info!("{:?}", action);

    // Act on the action
    match Action::act(&data, action) {
        Ok(ok) => {
            info!("{:?}", ok);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn init_cli() -> clap::ArgMatches {
    command!() // requires `cargo` feature
        .arg(arg!(
            -d --debug ... "Turn debugging information on"
        ))
        .subcommand(
            Command::new("new")
                .about("Create a new note")
                .arg(arg!(-t --template "select a template").action(ArgAction::Set))
                .arg(arg!([name] "Note to operate on, or create if only arg given")),
        )
        .subcommand(
            Command::new("add")
                .about("Add an existing file to notes")
                .arg(arg!([name] "Full path to note to add")),
        )
        .subcommand(Command::new("templates").about("List existing templates"))
        .subcommand(Command::new("list").about("List all known notes"))
        // .subcommand(Command::new("search").about("Add an existing file to notes"))
        .arg(arg!([name] "Note to operate on, or create if only arg given"))
        .arg(arg!(-t --template "select a template").action(ArgAction::Set))
        .get_matches()
}

fn parse_name_arg(
    data: &Data,
    cli_result: &clap::ArgMatches,
    template: &mut Option<String>,
    name: &mut Option<String>,
) {
    if let Some(name_arg) = cli_result.get_one::<String>("name") {
        if template.is_none()
            && (Data::template_file_exists(name_arg) || data.get_template(name_arg).is_some())
        {
            info!("Value for name: {}, assumed to be template", name_arg);
            *template = Some(name_arg.to_owned());
        } else {
            info!("Value for name: {}", name_arg);
            *name = Some(name_arg.to_owned());
        }
    }
}

fn parse_template_arg(
    cli_result: &clap::ArgMatches,
    template: &mut Option<String>,
) -> Result<(), Box<dyn Error>> {
    if let Some(template_arg) = cli_result.get_one::<String>("template") {
        info!("Value for template: {}", template_arg);
        if Data::template_file_exists(template_arg) {
            *template = Some(template_arg.to_owned());
            return Ok(());
        } else {
            return Err(format!("No template named `{}`", template_arg).into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_new_() {
        assert_eq!(true, true);
    }
}
