use std::io::prelude::*;
use std::io::Result;
use std::path::PathBuf;

use clap::Parser;
use taxonomy_lookup::TaxonomyDatabaseConfig;

/// Look up accession numbers in the taxonomy database
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Where to find the database files. By default, this tool will look for them in
    /// "$XDG_DATA_HOME/taxonomy_lookup/".
    #[clap(short, long)]
    taxonomy_dir: Option<PathBuf>,

    /// Accept line-separated accession numbers form stdin as well as from the command
    /// line.
    #[clap(short, long)]
    stdin: bool,

    /// A list of accession numbers, e.g. U39076.1
    accession_numbers: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut config = TaxonomyDatabaseConfig::new().cache_size(10_000_000_000);
    if let Some(tax_dir) = args.taxonomy_dir {
        config = config.location(tax_dir);
    }

    let db = config.build()?;

    for an in args.accession_numbers {
        if let Ok(info) = db.query_accession(&an) {
            println!("{:?}", info);
        } else {
            eprintln!("Could not find {}", &an)
        }
    }

    if args.stdin {
        for an_maybe in std::io::stdin().lock().lines() {
            let an = an_maybe?;
            if let Ok(info) = db.query_accession(&an) {
                println!("{:?}", info);
            } else {
                eprintln!("Could not find {}", &an)
            }
        }
    }
    Ok(())
}
