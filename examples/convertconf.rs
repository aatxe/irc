use std::env;
use std::process::exit;

use args::{Args, ArgsError};
use getopts::Occur;
use irc::client::data::Config;

const PROGRAM_DESC: &str = "Use this program to convert configs between {JSON, TOML, YAML}.";
const PROGRAM_NAME: &str = "convertconf";

fn main() {
    let args: Vec<_> = env::args().collect();
    match parse(&args) {
        Ok(Some((ref input, ref output))) => {
            let mut cfg = Config::load(input).unwrap();
            cfg.save(output).unwrap();
            println!("Converted {} to {}.", input, output);
        }
        Ok(None) => {
            println!("Failed to provide required arguments.");
            exit(1);
        }
        Err(err) => {
            println!("{}", err);
            exit(1);
        }
    }
}

fn parse(input: &[String]) -> Result<Option<(String, String)>, ArgsError> {
    let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
    args.flag("h", "help", "Print the usage menu");
    args.option(
        "i",
        "input",
        "The path to the input config",
        "FILE",
        Occur::Req,
        None,
    );
    args.option(
        "o",
        "output",
        "The path to output the new config to",
        "FILE",
        Occur::Req,
        None,
    );

    args.parse(input)?;

    let help = args.value_of("help")?;
    if help {
        args.full_usage();
        return Ok(None);
    }

    Ok(Some((args.value_of("input")?, args.value_of("output")?)))
}
