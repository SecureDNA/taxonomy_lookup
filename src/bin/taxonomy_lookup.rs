use std::io::Result;
use std::path::PathBuf;

use clap::Parser;
use taxonomy_lookup::TaxonomyDatabaseConfig;

/// Look up accession numbers in the taxonomy database
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Where to find the database files. By default, this tool will look for them in
    /// "$XDG_DATA_HOME/taxonomy_lookup/"
    #[clap(short, long)]
    taxonomy_dir: Option<PathBuf>,

    accession_numbers: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut config = TaxonomyDatabaseConfig::new();
    if let Some(tax_dir) = args.taxonomy_dir {
        config = config.location(tax_dir);
    }

    let db = config.build()?;

    for an in args.accession_numbers {
        let info = db.query_accession(&an)?;
        println!("{:?}", info);
    }
    Ok(())
}
