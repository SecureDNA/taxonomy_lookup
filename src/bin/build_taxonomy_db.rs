use std::path::PathBuf;

use clap::Parser;

use taxonomy_lookup::{TaxonomyDatabaseConfig, TaxonomyDatabaseSource};

/// Produce the taxonomy database
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Directory containing NCBI taxonomy information. This should be pulled from
    /// ftp.ncbi.nih.gov/pub/taxonomy/. It should include a file `taxdump.tar.gz`,
    /// and a directory `accession2taxid/` containing the files `prot.accession2taxid.gz`,
    /// `nucl_wgs.accession2taxid.gz`, and `nucl_gb.accession2taxid.gz`.
    taxonomy_dir: PathBuf,

    /// Where to put the resulting database files. By default, this tool will place them in
    /// "$XDG_DATA_HOME/taxonomy_lookup/", which is where the library expects to find them by
    /// default.
    output_filename: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let mut config = TaxonomyDatabaseConfig::new()
        .source(TaxonomyDatabaseSource::FromFiles(args.taxonomy_dir.clone()));
    config = if let Some(p) = args.output_filename {
        config.location(p)
    } else {
        config
    };
    config.build()?;

    Ok(())
}
