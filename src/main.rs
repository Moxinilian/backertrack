use chrono::{TimeZone, Utc};
use clap::{App, AppSettings, Arg, SubCommand};
use num::{BigInt, BigRational};

use std::path::PathBuf;

pub const DATE_FORMAT: &'static str = "%Y/%m/%d %H:%M";

mod ledger;
mod utils;

#[cfg(unix)]
mod ui;

#[cfg(windows)]
mod ui {
    pub fn start_handle_panic(
        _: Option<std::path::PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("backertrack UI is only supported on Unix systems.");
        Ok(())
    }
}

use crate::utils::path_exists_or_panic;

fn main() -> Result<(), Box<std::error::Error>> {
    let ledger_subcommand = SubCommand::with_name("ledger")
        .about("Manage the transaction ledger")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("new")
                .about("Create a blank ledger")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::with_name("LEDGER")
                        .required(true)
                        .help("Path to create the ledger file at"),
                ),
        )
        .subcommand(
            SubCommand::with_name("export")
                .about("Export the ledger to an accountant-friendly format")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::with_name("LEDGER")
                        .required(true)
                        .help("Path where the ledger lives"),
                )
                .arg(
                    Arg::with_name("OUTPUT")
                        .required(true)
                        .help("Path to send the output CSV to"),
                ),
        )
        .subcommand(
            SubCommand::with_name("donations")
                .about("Manage donations")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("import")
                        .about("Import new donations into the ledger")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .arg(
                            Arg::with_name("LEDGER")
                                .required(true)
                                .help("Path to the ledger file"),
                        )
                        .arg(
                            Arg::with_name("FILE")
                                .required(true)
                                .help("Path to the file to import data from"),
                        )
                        .arg(
                            Arg::with_name("FORMAT")
                                .required(true)
                                .help("Format of the imported data")
                                .possible_values(&["donorbox", "opencollective"]),
                        ),
                ),
        )
        .subcommand(
            SubCommand::with_name("accounts")
                .about("Manage ledger accounts")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("new")
                        .about("Create a new account in the ledger")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .arg(
                            Arg::with_name("LEDGER")
                                .required(true)
                                .help("Path to the ledger file"),
                        )
                        .arg(
                            Arg::with_name("NAME")
                                .required(true)
                                .help("Name of the new account"),
                        )
                        .arg(Arg::with_name("DATE").required(true).help("Date the account was opened (YYYY/MM/DD HH:MM)"))
                        .arg(
                            Arg::with_name("balance")
                                .long("balance")
                                .short("b")
                                .takes_value(true)
                                .help("Opening balance for the account"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete an account from the ledger")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .arg(
                            Arg::with_name("LEDGER")
                                .required(true)
                                .help("Path to the ledger file"),
                        )
                        .arg(
                            Arg::with_name("NAME")
                                .required(true)
                                .help("Name of the account to delete"),
                        )
                        .arg(
                            Arg::with_name("force")
                                .short("F")
                                .help("Bypass all warnings"),
                        ),
                ),
        );

    let app = App::new("backertrack")
        .author("The Amethyst Project Developers")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Various tools to manage the Amethyst Foundation's treasury paperwork")
        .arg(
            Arg::with_name("ledger")
                .short("l")
                .takes_value(true)
                .help("Path to the ledger file"),
        )
        .subcommand(ledger_subcommand);

    let matches = app.get_matches();

    if let Some(ledger_match) = matches.subcommand_matches("ledger") {
        if let Some(donations_match) = ledger_match.subcommand_matches("donations") {
            if let Some(import_match) = donations_match.subcommand_matches("import") {
                ledger::donations::import(
                    path_exists_or_panic(import_match.value_of("LEDGER").unwrap()),
                    path_exists_or_panic(import_match.value_of("FILE").unwrap()),
                    import_match.value_of("FORMAT").unwrap().into(),
                );
            }
        } else if let Some(accounts_match) = ledger_match.subcommand_matches("accounts") {
            if let Some(new_match) = accounts_match.subcommand_matches("new") {
                ledger::accounts::new(
                    path_exists_or_panic(new_match.value_of("LEDGER").unwrap()),
                    new_match.value_of("NAME").unwrap(),
                    new_match
                        .value_of("balance")
                        .map(std::str::FromStr::from_str)
                        .and_then(Result::ok)
                        .unwrap_or_else(|| BigRational::from_integer(BigInt::from(0))),
                    new_match
                        .value_of("DATE")
                        .and_then(|x| Utc.datetime_from_str(x, DATE_FORMAT).ok())
                        .expect("Invalid opening date, expected format: YYYY/MM/DD HH:MM"),
                );
            }
        } else if let Some(new_match) = ledger_match.subcommand_matches("new") {
            ledger::new(PathBuf::from(new_match.value_of("LEDGER").unwrap()));
        } else if let Some(export_match) = ledger_match.subcommand_matches("export") {
            ledger::export(
                PathBuf::from(export_match.value_of("LEDGER").unwrap()),
                PathBuf::from(export_match.value_of("OUTPUT").unwrap()),
            );
        }
    } else {
        let ledger = matches.value_of("ledger").map(PathBuf::from);
        ui::start_handle_panic(ledger)?;
    }

    Ok(())
}
