use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use clap::Parser;
use flate2::read::GzDecoder;
use tar::Archive;

use taxonomy_lookup::Rank;

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

fn read_names_file<R: Read>(f: R) -> io::Result<BTreeMap<u32, String>> {
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
        let taxon = fields[0]
            .parse::<u32>()
            .map_err(|_| io::Error::new(InvalidData, "Invalid taxon ID in names.dmp"))?;
        result.insert(taxon, String::from(fields[2]));
    }
    Ok(result)
}

fn read_nodes_file<R: Read>(f: R) -> io::Result<BTreeMap<u32, (u32, Rank)>> {
    use io::ErrorKind::InvalidData;

    let mut result = BTreeMap::new();
    for l in BufReader::new(f).lines() {
        let line = l?;
        let field_iter = line.split('\t');
        let fields = field_iter.collect::<Vec<&str>>();
        if fields.len() != 26 {
            return Err(io::Error::new(InvalidData, "Invalid line in nodes.dmp"));
        }
        let taxon = fields[0]
            .parse::<u32>()
            .map_err(|_| io::Error::new(InvalidData, "Invalid taxon ID in nodes.dmp"))?;
        let parent = fields[2]
            .parse::<u32>()
            .map_err(|_| io::Error::new(InvalidData, "Invalid taxon ID in nodes.dmp"))?;
        // ranks.insert(fields[4].to_owned());
        let rank = fields[4].parse::<Rank>().map_err(|_| {
            io::Error::new(
                InvalidData,
                format!("Invalid rank in nodes.dmp: {:?}", fields[4]),
            )
        })?;
        result.insert(taxon, (parent, rank));
    }
    Ok(result)
}

fn read_accessions_to_db<R: Read>(f: R, db: &sled::Tree) -> io::Result<()> {
    use io::ErrorKind::InvalidData;
    let mut lines = BufReader::new(f).lines();
    let first_line = lines.next().unwrap_or(Err(io::Error::new(
        InvalidData,
        format!("Empty accessions2taxid file"),
    )))?;

    let headers = first_line.split('\t').collect::<Vec<&str>>();
    let accession_column = headers
        .iter()
        .position(|&s| s == "accession")
        .ok_or(io::Error::new(
            InvalidData,
            format!("accessions2taxid file missing accession column"),
        ))?;
    let taxid_column = headers
        .iter()
        .position(|&s| s == "taxid")
        .ok_or(io::Error::new(
            InvalidData,
            format!("accessions2taxid file missing taxid column"),
        ))?;

    let mut last_taxid_and_accession: Option<(u32, String)> = None;

    for l in lines {
        let line = l?;
        let field_iter = line.split('\t');
        let fields = field_iter.map(|s| s.to_string()).collect::<Vec<String>>();
        let taxid = fields[taxid_column].parse::<u32>()
            .map_err(|_| io::Error::new(InvalidData, "Invalid taxon ID in accessions map file"))?;
        match last_taxid_and_accession {
            Some((last_taxid, last_acc)) if last_taxid != taxid => {
                db.insert(last_acc, &last_taxid.to_le_bytes())?;
                db.insert(&fields[accession_column], &taxid.to_le_bytes())?;
            }
            None => {
                db.insert(&fields[accession_column], &taxid.to_le_bytes())?;
            }
            _ => {}
        }
        last_taxid_and_accession = Some((taxid, fields[accession_column].clone()));
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let taxdump_file = File::open(args.taxonomy_dir.join("taxdump.tar.gz"))?;
    let taxdump_gz = GzDecoder::new(&taxdump_file);
    let mut taxdump_archive = Archive::new(taxdump_gz);
    let mut names: BTreeMap<u32, String> = BTreeMap::new();
    let mut node_tree: BTreeMap<u32, (u32, Rank)> = BTreeMap::new();
    for e in taxdump_archive.entries()? {
        let entry = e?;
        if entry.path()? == Path::new("names.dmp") {
            names = read_names_file(entry)?;
        } else if entry.path()? == Path::new("nodes.dmp") {
            node_tree = read_nodes_file(entry)?;
        }
    }

    let taxa_map_dir_path = args.taxonomy_dir.join("accession2taxid");

    let db_path = if let Some(p) = args.output_filename { p } else {
        xdg::BaseDirectories::with_prefix("taxonomy_lookup")?.place_data_file("taxonomy.sled")?
    };

    let _ = std::fs::remove_dir_all(&db_path);

    let db = sled::Config::default()
        .path(&db_path)
        .mode(sled::Mode::LowSpace)
        .use_compression(false)
        // .compression_factor(22)
        .create_new(true)
        .open()?;

    let taxa_map_dir = std::fs::read_dir(taxa_map_dir_path)?;

    for f in taxa_map_dir {
        let path = f?.path();
        if path.to_str().unwrap().ends_with("gz") {
            let taxa_zip_mapped = File::open(path)?;
            let taxa_file = GzDecoder::new(&taxa_zip_mapped);
             read_accessions_to_db(taxa_file, &db.open_tree("accession_map")?)?;
        }
    }
    let name_map_db = db.open_tree("name_map")?;
    for (k, v) in names.iter() {
        name_map_db.insert(k.to_le_bytes(), v.as_str())?;
    }

    let node_tree_db = db.open_tree("node_tree")?;
    let node_ranks_db = db.open_tree("node_ranks")?;
    for (k, (parent, rank)) in node_tree {
        node_tree_db.insert(&k.to_le_bytes(), &parent.to_le_bytes())?;
        node_ranks_db.insert(&k.to_le_bytes(), &[rank as u8])?;
    }

    db.flush()?;

    Ok(())
}
