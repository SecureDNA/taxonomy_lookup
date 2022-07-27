use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;

use crate::rank::Rank;

fn data_error(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

pub enum TaxonomyDatabaseSource {
    FromExisting,
    FromGzipped(std::path::PathBuf),
    // FromGzippedUrl(url::Url),
    FromFiles(std::path::PathBuf),
    // FromFilesUrl(url::Url)
}

pub struct TaxonomyDatabaseConfig {
    source: TaxonomyDatabaseSource,
    cache_size: Option<u64>,
    location: Option<std::path::PathBuf>,
}

fn read_names_file<R: Read>(f: R) -> io::Result<BTreeMap<u32, String>> {
    let mut result = BTreeMap::new();
    for l in BufReader::new(f).lines() {
        let line = l?;
        let fields = line.split('\t').collect::<Vec<&str>>();
        if fields.len() != 8 {
            return Err(data_error("Invalid line in names.dmp"));
        }
        if fields[6] != "scientific name" {
            continue;
        }
        let taxon = fields[0]
            .parse::<u32>()
            .map_err(|_| data_error("Invalid taxon ID in names.dmp"))?;
        result.insert(taxon, String::from(fields[2]));
    }
    Ok(result)
}

fn read_nodes_file<R: Read>(f: R) -> io::Result<BTreeMap<u32, (u32, Rank)>> {
    let mut result = BTreeMap::new();
    for l in BufReader::new(f).lines() {
        let line = l?;
        let field_iter = line.split('\t');
        let fields = field_iter.collect::<Vec<&str>>();
        if fields.len() != 26 {
            return Err(data_error("Invalid line in nodes.dmp"));
        }
        let taxon = fields[0]
            .parse::<u32>()
            .map_err(|_| data_error("Invalid taxon ID in nodes.dmp"))?;
        let parent = fields[2]
            .parse::<u32>()
            .map_err(|_| data_error("Invalid taxon ID in nodes.dmp"))?;
        // ranks.insert(fields[4].to_owned());
        let rank = fields[4].parse::<Rank>().map_err(|_| {
            let msg = format!("Invalid rank in nodes.dmp: {:?}", fields[4]);
            data_error(&msg)
        })?;
        result.insert(taxon, (parent, rank));
    }
    Ok(result)
}

fn read_accessions<R: Read>(
    fs: impl Iterator<Item = R>,
) -> io::Result<impl Iterator<Item = (String, u32)>> {
    let mut pair_iters = vec![];
    for f in fs {
        let mut lines = BufReader::new(f).lines();
        let first_line = lines
            .next()
            .unwrap_or_else(|| Err(data_error("Empty accessions2taxid file")))?;
        let headers = first_line.split('\t').collect::<Vec<&str>>();
        let accession_column = headers
            .iter()
            .position(|&s| s == "accession" || s == "accession.version")
            .ok_or_else(|| data_error("accessions2taxid file missing accession column"))?;
        let taxid_column = headers
            .iter()
            .position(|&s| s == "taxid")
            .ok_or_else(|| data_error("accessions2taxid file missing taxid column"))?;
        let pair_iter = lines.filter_map(move |l| {
            let line = l.ok()?;
            let field_iter = line.split('\t');
            let fields = field_iter.map(|s| s.to_string()).collect::<Vec<String>>();
            let taxid = fields[taxid_column].parse::<u32>().ok()?;
            let accession = &fields[accession_column];
            let bare_acc: String = accession.split('.').next().unwrap().to_string();
            Some((bare_acc, taxid))
        });
        pair_iters.push(pair_iter)
    }

    use itertools::kmerge;
    Ok(kmerge(pair_iters))
}

fn read_accessions_to_db(
    pairs: impl Iterator<Item = (String, u32)>,
    db: &sled::Tree,
) -> io::Result<()> {
    let mut last_accession_and_taxid: Option<(String, u32)> = None;
    let mut last_insertion: Option<(String, u32)> = None;

    for pair in pairs {
        let (accession, taxid) = pair;

        // Under what conditions do we want to insert an accession:taxid mapping?
        //
        // First, we want to totally ignore any duplicated accessions. If we have already seen
        // an accession, we ignore all subsequent ones.
        //
        // Having dealt with this possibility, we want to insert the "endstops" of every run of
        // shared taxids. We do this by looking for transitions between different taxids, and
        // inserting the mappings from both sides of the transition.

        let insertions: Vec<(String, u32)> = match (&last_accession_and_taxid, &last_insertion) {
            (None, _) => vec![(accession.clone(), taxid)],
            (Some((last_acc, last_taxid)), Some((_, last_inserted_taxid))) => {
                if last_acc == &accession {
                    // If this accession is the same as the last one we saw, we want to skip it
                    // entirely. Thus whenever we see multiple accessions, we always choose the
                    // first taxid we see.
                    continue;
                } else if taxid == *last_inserted_taxid {
                    // Similarly, if we most recently inserted an item with a given taxid, we're in
                    // a run. If this is the last element of the run we'll insert it on the next
                    // iteration; if it isn't the last run element we want to skip it.
                    vec![]
                } else {
                    // finally we know that we've achieved a proper transition between two runs.
                    vec![(last_acc.clone(), *last_taxid), (accession.clone(), taxid)]
                }
            }
            _ => vec![],
        };

        for (acc_insert, taxid_insert) in insertions {
            db.insert(acc_insert.clone(), &taxid_insert.to_le_bytes())?;
            last_insertion = Some((acc_insert.clone(), taxid_insert));
        }
        last_accession_and_taxid = Some((accession, taxid));
    }
    if let Some((last_acc, last_taxid)) = last_accession_and_taxid {
        db.insert(last_acc, &last_taxid.to_le_bytes())?;
    }
    Ok(())
}

const ACCESSION_TO_TAXON: &str = "accession_to_taxon";
const TAXON_TO_NAME: &str = "taxon_to_name";
const TAXON_TREE: &str = "taxon_tree";
const TAXON_RANKS: &str = "taxon_ranks";
const TAXONOMY_DB_VERSION_KEY: &[u8] = b"taxonomy_db_version";
const TAXONOMY_DB_VERSION: &[u8] = b"1";

fn build_new_db(db: sled::Db, source_path: &Path) -> io::Result<TaxonomyDatabase> {
    let taxdump_file = File::open(source_path.join("taxdump.tar.gz"))?;
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

    let taxa_map_dir_path = source_path.join("accession2taxid");

    let taxa_map_dir = std::fs::read_dir(taxa_map_dir_path)?;
    let fs_iter = taxa_map_dir.filter_map(|f| {
        let path = f.ok()?.path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with("accession2taxid.gz")
            || path_str.ends_with("accession2taxid.FULL.gz")
            || path_str.ends_with("accession2taxid.EXTRA.gz")
        {
            Some(GzDecoder::new(File::open(path).ok()?))
        } else {
            None
        }
    });

    let accessions = db.open_tree(ACCESSION_TO_TAXON)?;

    read_accessions_to_db(read_accessions(fs_iter)?, &accessions)?;
    let name_map_db = db.open_tree(TAXON_TO_NAME)?;
    for (k, v) in names.iter() {
        name_map_db.insert(k.to_le_bytes(), v.as_str())?;
    }

    let node_tree_db = db.open_tree(TAXON_TREE)?;
    let node_ranks_db = db.open_tree(TAXON_RANKS)?;
    for (k, (parent, rank)) in node_tree {
        node_tree_db.insert(&k.to_le_bytes(), &parent.to_le_bytes())?;
        node_ranks_db.insert(&k.to_le_bytes(), &[rank as u8])?;
    }
    db.insert(TAXONOMY_DB_VERSION_KEY, TAXONOMY_DB_VERSION)?;

    db.flush()?;
    Ok(TaxonomyDatabase {
        accession_to_taxon: accessions,
        taxon_to_name: name_map_db,
        taxon_tree: node_tree_db,
        taxon_ranks: node_ranks_db,
    })
}

pub fn unzip_db<Ps: AsRef<Path>, Pt: AsRef<Path>>(source: Ps, target: Pt) -> io::Result<()> {
    let source_file = File::open(source)?;
    let source_gz = GzDecoder::new(&source_file);
    let mut archive = Archive::new(source_gz);
    archive.unpack(target)
}

fn open_existing(db_config: sled::Config) -> io::Result<TaxonomyDatabase> {
    let db = db_config.open()?;
    if let Ok(Some(v)) = db.get(TAXONOMY_DB_VERSION_KEY) {
        if &(*v) != TAXONOMY_DB_VERSION {
            return Err(data_error("Taxonomy database has incompatible version"));
        }
    }
    Ok(TaxonomyDatabase {
        accession_to_taxon: db.open_tree(ACCESSION_TO_TAXON)?,
        taxon_to_name: db.open_tree(TAXON_TO_NAME)?,
        taxon_tree: db.open_tree(TAXON_TREE)?,
        taxon_ranks: db.open_tree(TAXON_RANKS)?,
    })
}

impl TaxonomyDatabaseConfig {
    pub fn new() -> Self {
        TaxonomyDatabaseConfig {
            source: TaxonomyDatabaseSource::FromExisting,
            cache_size: None,
            location: None,
        }
    }

    pub fn cache_size(mut self, size: u64) -> Self {
        self.cache_size = Some(size);
        self
    }

    pub fn location(mut self, location: std::path::PathBuf) -> Self {
        self.location = Some(location);
        self
    }

    pub fn source(mut self, source: TaxonomyDatabaseSource) -> Self {
        self.source = source;
        self
    }

    pub fn build(&self) -> std::io::Result<TaxonomyDatabase> {
        let db_path = if let Some(ref p) = &self.location {
            p.to_owned()
        } else {
            xdg::BaseDirectories::with_prefix("taxonomy_lookup")?
                .place_data_file("taxonomy.sled")?
        };

        let mut db_config = sled::Config::default()
            .path(&db_path)
            .mode(sled::Mode::LowSpace);

        if let Some(cache_size) = self.cache_size {
            db_config = db_config.cache_capacity(cache_size);
        }

        Ok(match &self.source {
            TaxonomyDatabaseSource::FromFiles(ref path) => {
                let _ = std::fs::remove_dir_all(&db_path);
                let db = db_config.open()?;
                build_new_db(db, path)?
            }
            TaxonomyDatabaseSource::FromExisting => open_existing(db_config)?,
            TaxonomyDatabaseSource::FromGzipped(ref path) => {
                unzip_db(path, &db_path)?;
                open_existing(db_config)?
            }
        })
    }
}

impl Default for TaxonomyDatabaseConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TaxonomyDatabase {
    accession_to_taxon: sled::Tree,
    taxon_to_name: sled::Tree,
    taxon_tree: sled::Tree,
    taxon_ranks: sled::Tree,
}

#[derive(Debug)]
pub struct TaxonomyInfo(pub Vec<(Rank, String)>);

// TODO throughout here, there's a bunch of annoying repetitive error stuff. Probably
// I should just make this less bad somehow.
impl TaxonomyDatabase {
    pub fn rank(&self, taxon: u32) -> std::io::Result<Rank> {
        let content = self.taxon_ranks.get(taxon.to_le_bytes())?.ok_or_else(|| {
            data_error("Corrupted taxonomy rank information: Could not find node")
        })?;
        let rank_bytes: [u8; 1] = (*content).try_into().map_err(|_| {
            data_error("Corrupted taxonomy rank information: Could not convert to bytes")
        })?;
        rank_bytes[0].try_into().map_err(|_| {
            data_error("Corrupted taxonomy rank information: Could not convert to enum")
        })
    }

    pub fn name(&self, taxon: u32) -> std::io::Result<String> {
        let content = self
            .taxon_to_name
            .get(taxon.to_le_bytes())?
            .ok_or_else(|| data_error("Corrupted taxonomy name information: node not found"))?;
        String::from_utf8((*content).try_into().map_err(|_| {
            data_error("Corrupted taxonomy name information: could not convert to bytes")
        })?)
        .map_err(|_| data_error("Corrupted taxonomy name information: invalid utf8"))
    }

    pub fn query_taxon(&self, taxon: u32) -> std::io::Result<TaxonomyInfo> {
        let taxon_bytes = taxon.to_le_bytes();
        let mut ancestor_taxons = vec![];
        let mut ancestor_id = taxon_bytes;
        while ancestor_id != 1u32.to_le_bytes() {
            ancestor_taxons.push(u32::from_le_bytes(ancestor_id));
            if let Ok(Some(content)) = self.taxon_tree.get(ancestor_id) {
                ancestor_id = (*content).try_into().map_err(|_| {
                    data_error(
                        "Corrupted taxonomy node information: could not read ancestor id bytes",
                    )
                })?
            } else {
                return Err(data_error(
                    "Corrupted taxonomy node information: could not find ancestor",
                ));
            }
        }

        let mut result = vec![];

        for taxon in ancestor_taxons {
            result.push((self.rank(taxon)?, self.name(taxon)?));
        }

        Ok(TaxonomyInfo(result))
    }

    pub fn query_accession(&self, accession: &str) -> std::io::Result<TaxonomyInfo> {
        let bare_acc = accession.split('.').next().unwrap().as_bytes();

        let taxon_vec = if let Some(node) = self.accession_to_taxon.get(&bare_acc)? {
            node
        } else {
            match (
                self.accession_to_taxon.get_lt(&bare_acc)?,
                self.accession_to_taxon.get_gt(&bare_acc)?,
            ) {
                (Some((_, lbs)), Some((_, rbs))) if lbs == rbs => lbs,
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Accession number not found in database",
                    ))
                }
            }
        };

        let taxon_bytes: [u8; 4] = (*taxon_vec).try_into().map_err(|_| {
            data_error("Corrupted taxonomy node information: Could not get taxon bytes")
        })?;

        self.query_taxon(u32::from_le_bytes(taxon_bytes))
    }
}
