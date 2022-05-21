use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, BufReader, BufRead};
use std::path::{Path, PathBuf};

use clap::Parser;
use flate2::read::GzDecoder;
use tar::Archive;

use taxonomy_lookup::rank::Rank;

/// Produce the taxonomy database
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Directory containing NCBI taxonomy information. This should be pulled from
    /// ftp.ncbi.nih.gov/pub/taxonomy/. It should include a file `taxdump.tar.gz`,
    /// and a directory `accession2taxid/` containing the files `prot.accession2taxid.gz`,
    /// `nucl_wgs.accession2taxid.gz`, and `nucl_gb.accession2taxid.gz`.
    taxonomy_dir: PathBuf,

    /// Where to put the resulting database file. By default, this tool will place it in
    /// "$XDG_DATA_HOME/taxonomy.db", which is where the library expects to find it.
    output_filename: Option<PathBuf>,
}


#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Taxon(u32);

fn read_names_file<R: Read>(f: R) -> io::Result<BTreeMap<Taxon, String>> {
    use io::ErrorKind::InvalidData;

    let mut result = BTreeMap::new();
    for l in BufReader::new(f).lines() {
        let line = l?;
        let fields = line.split('\t').collect::<Vec<&str>>();
        if fields.len() != 8 {
            return Err(io::Error::new(InvalidData, "Invalid line in names.dmp"));
        }
        if fields[6] != "scientific name" {
            continue;
        }
        let taxon = Taxon(
            fields[0].parse::<u32>()
            .map_err(|_| io::Error::new(InvalidData, "Invalid taxon ID in names.dmp"))?);
        result.insert(taxon, String::from(fields[2]));
    }
    Ok(result)
}

fn read_nodes_file<R: Read>(f: R) -> io::Result<BTreeMap<Taxon, (Taxon, Rank)>> {
    use io::ErrorKind::InvalidData;

    let mut result = BTreeMap::new();
    for l in BufReader::new(f).lines() {
        let line = l?;
        let field_iter = line.split('\t');
        let fields = field_iter.collect::<Vec<&str>>();
        if fields.len() != 26 {
            return Err(io::Error::new(InvalidData, "Invalid line in nodes.dmp"));
        }
        let taxon = Taxon(
            fields[0].parse::<u32>()
            .map_err(|_| io::Error::new(InvalidData, "Invalid taxon ID in nodes.dmp"))?);
        let parent = Taxon(
            fields[2].parse::<u32>()
            .map_err(|_| io::Error::new(InvalidData, "Invalid taxon ID in nodes.dmp"))?);
        // ranks.insert(fields[4].to_owned());
        let rank = fields[4].parse::<Rank>()
            .map_err(|_| io::Error::new(InvalidData, format!("Invalid rank in nodes.dmp: {:?}", fields[4])))?;
        result.insert(taxon, (parent, rank));
    }
    println!("{:?}", result.len());
    Ok(result)
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let taxdump_file = File::open(args.taxonomy_dir.join("taxdump.tar.gz"))?;
    let taxdump_gz = GzDecoder::new(&taxdump_file);
    let mut taxdump_archive = Archive::new(taxdump_gz);
    let mut names: BTreeMap<Taxon, String> = BTreeMap::new();
    let mut node_tree: BTreeMap<Taxon, (Taxon, Rank)> = BTreeMap::new();
    for e in taxdump_archive.entries()? {
        let entry = e?;
        if entry.path()? == Path::new("names.dmp") {
            names = read_names_file(entry)?;
        } else if entry.path()? == Path::new("nodes.dmp") {
            node_tree = read_nodes_file(entry)?;
        }
        // println!("{:?}", e?.path()?.into_owned());
    }

    Ok(())
}
