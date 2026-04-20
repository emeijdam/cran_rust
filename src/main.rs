//use rds2rust::{RObject, read_rds};
use std::{fs::File, io::Write, path::{ PathBuf}};
use clap::{Parser, Subcommand};

use crate::{dcf::parse_dcf, rds::parse_rds_file};

mod dcf;
mod rds;

#[derive(Parser)]
#[command(name = "rparse")]
#[command(about = "A utility to parse R-related file formats

A DCF file in the context of CRAN stands for Debian Control File, 
a plain-text format used to store metadata about R packages. 

The most common example is the DESCRIPTION file within a package, 
which holds crucial info like package name, version, dependencies, and author details.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse a DCF file into JSON
    Dcf {
        /// Sets the input DCF file
        input: PathBuf,
        /// Sets the output JSON file
        output: PathBuf,
    },
    /// Parse an RDS file into CSV
    Rds {
        /// Sets the input RDS file
        input: PathBuf,
        /// Sets the output CSV file
        output: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match &args.command {
        Commands::Dcf { input, output } => {
            println!("Parsing DCF: {:?} -> {:?}", input, output);
            
            let dcf = parse_dcf(input)?;
            let json_string = serde_json::to_string_pretty(&dcf)?;
            let mut file = File::create(output)?;
            file.write_all(json_string.as_bytes())?;
            println!("Successfully wrote JSON to {:?}", output);
        }
        Commands::Rds { input, output } => {
            println!("Parsing RDS: {:?} -> {:?}", input, output);
            parse_rds_file(input, output)?;
            println!("Successfully wrote CSV to {:?}", output);
        }
    }

    Ok(())
}
