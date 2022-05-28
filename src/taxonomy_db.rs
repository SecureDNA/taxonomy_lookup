use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::str::FromStr;

use flate2::read::GzDecoder;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tar::Archive;

/// Geez NCBI, make up your mind.
#[derive(Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Rank {
    NoRank,

    Clade,

    Superkingdom,
    Kingdom,
    Subkingdom,

    Superphylum,
    Phylum,
    Subphylum,

    Superclass,
    Class,
    Subclass,
    Infraclass,

    Cohort,
    Subcohort,

    Superorder,
    Order,
    Suborder,
    Infraorder,
    Parvorder,

    Superfamily,
    Family,
    Subfamily,

    Tribe,
    Subtribe,

    Genus,
    Subgenus,

    Section,
    Subsection,
    Series,

    SpeciesGroup,
    SpeciesSubgroup,
    Species,
    Subspecies,

    Morph,
    Varietas,
    Forma,
    FormaSpecialis,
    Pathogroup,
    Strain,
    Serogroup,
    Serotype,
    Genotype,
    Biotype,
    Isolate,
}

impl FromStr for Rank {
    type Err = ();
    fn from_str(input: &str) -> Result<Rank, Self::Err> {
        match input {
            "no rank" => Ok(Rank::NoRank),
            "clade" => Ok(Rank::Clade),
            "superkingdom" => Ok(Rank::Superkingdom),
            "kingdom" => Ok(Rank::Kingdom),
            "subkingdom" => Ok(Rank::Subkingdom),
            "superphylum" => Ok(Rank::Superphylum),
            "phylum" => Ok(Rank::Phylum),
            "subphylum" => Ok(Rank::Subphylum),
            "superclass" => Ok(Rank::Superclass),
            "class" => Ok(Rank::Class),
            "subclass" => Ok(Rank::Subclass),
            "infraclass" => Ok(Rank::Infraclass),
            "cohort" => Ok(Rank::Cohort),
            "subcohort" => Ok(Rank::Subcohort),
            "superorder" => Ok(Rank::Superorder),
            "order" => Ok(Rank::Order),
            "suborder" => Ok(Rank::Suborder),
            "infraorder" => Ok(Rank::Infraorder),
            "parvorder" => Ok(Rank::Parvorder),
            "superfamily" => Ok(Rank::Superfamily),
            "family" => Ok(Rank::Family),
            "subfamily" => Ok(Rank::Subfamily),
            "tribe" => Ok(Rank::Tribe),
            "subtribe" => Ok(Rank::Subtribe),
            "genus" => Ok(Rank::Genus),
            "subgenus" => Ok(Rank::Subgenus),
            "section" => Ok(Rank::Section),
            "subsection" => Ok(Rank::Subsection),
            "series" => Ok(Rank::Series),
            "species group" => Ok(Rank::SpeciesGroup),
            "species subgroup" => Ok(Rank::SpeciesSubgroup),
            "species" => Ok(Rank::Species),
            "subspecies" => Ok(Rank::Subspecies),
            "morph" => Ok(Rank::Morph),
            "varietas" => Ok(Rank::Varietas),
            "forma" => Ok(Rank::Forma),
            "forma specialis" => Ok(Rank::FormaSpecialis),
            "pathogroup" => Ok(Rank::Pathogroup),
            "strain" => Ok(Rank::Strain),
            "serogroup" => Ok(Rank::Serogroup),
            "serotype" => Ok(Rank::Serotype),
            "genotype" => Ok(Rank::Genotype),
            "biotype" => Ok(Rank::Biotype),
            "isolate" => Ok(Rank::Isolate),
            _ => Err(()),
        }
    }
}

pub enum TaxonomyDatabaseSource {
    FromExisting,
    // FromGzipped(std::path::PathBuf),
    // FromGzippedUrl(url::Url),
    FromFiles(std::path::PathBuf),
    // FromFilesUrl(url::Url)
}

pub struct TaxonomyDatabaseConfig {
    source: TaxonomyDatabaseSource,
    location: Option<std::path::PathBuf>,
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

fn read_accessions<R: Read>(
    fs: impl Iterator<Item = R>,
) -> io::Result<impl Iterator<Item = (String, u32)>> {
    use io::ErrorKind::InvalidData;

    let mut pair_iters = vec![];
    for f in fs {
        let mut lines = BufReader::new(f).lines();
        let first_line = lines.next().unwrap_or(Err(io::Error::new(
            InvalidData,
            format!("Empty accessions2taxid file"),
        )))?;
        let headers = first_line.split('\t').collect::<Vec<&str>>();
        let accession_column =
            headers
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
        let pair_iter = lines.filter_map(move |l| {
            let line = l.ok()?;
            let field_iter = line.split('\t');
            let fields = field_iter.map(|s| s.to_string()).collect::<Vec<String>>();
            let taxid = fields[taxid_column].parse::<u32>().ok()?;
            let accession = fields[accession_column].clone();
            Some((accession, taxid))
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
    let mut last_taxid_and_accession: Option<(u32, String)> = None;

    for pair in pairs {
        let (accession, taxid) = pair;

        match last_taxid_and_accession {
            Some((last_taxid, last_acc)) if last_taxid != taxid => {
                db.insert(last_acc, &last_taxid.to_le_bytes())?;
                db.insert(&accession, &taxid.to_le_bytes())?;
            }
            None => {
                db.insert(&accession, &taxid.to_le_bytes())?;
            }
            _ => {}
        }
        last_taxid_and_accession = Some((taxid, accession.clone()));
    }
    Ok(())
}

impl TaxonomyDatabaseConfig {
    pub fn new() -> Self {
        TaxonomyDatabaseConfig {
            source: TaxonomyDatabaseSource::FromExisting,
            location: None,
        }
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

        let db_config = sled::Config::default()
            .path(&db_path)
            .mode(sled::Mode::LowSpace)
            .create_new(true);

        Ok(match &self.source {
            TaxonomyDatabaseSource::FromFiles(ref path) => {
                let taxdump_file = File::open(path.join("taxdump.tar.gz"))?;
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

                let taxa_map_dir_path = path.join("accession2taxid");

                let _ = std::fs::remove_dir_all(&db_path);

                let db = db_config.open()?;

                let taxa_map_dir = std::fs::read_dir(taxa_map_dir_path)?;
                let fs_iter = taxa_map_dir.filter_map(|f| {
                    let path = f.ok()?.path();
                    if path.to_str().unwrap().ends_with("gz") {
                        Some(GzDecoder::new(File::open(path).ok()?))
                    } else {
                        None
                    }
                });

                let accessions = db.open_tree("accession_map")?;

                read_accessions_to_db(read_accessions(fs_iter)?, &accessions)?;
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
                TaxonomyDatabase {
                    accession_map: accessions,
                    name_map: node_tree_db,
                    node_map: node_ranks_db,
                    node_ranks: name_map_db,
                }
            }
            TaxonomyDatabaseSource::FromExisting => {
                let db = db_config.open()?;
                TaxonomyDatabase {
                    accession_map: db.open_tree("accession_map")?,
                    name_map: db.open_tree("node_tree")?,
                    node_map: db.open_tree("node_ranks")?,
                    node_ranks: db.open_tree("name_map")?,
                }
            }
        })
    }
}

pub struct TaxonomyDatabase {
    accession_map: sled::Tree,
    name_map: sled::Tree,
    node_map: sled::Tree,
    node_ranks: sled::Tree,
}

pub struct TaxonomyInfo(Vec<(Rank, String)>);

// TODO throughout here, there's a bunch of annoying repetitive error stuff. Probably
// I should just make this less bad.
impl TaxonomyDatabase {
    pub fn rank(&self, taxon: u32) -> std::io::Result<Rank> {
        let content = self
            .node_ranks
            .get(taxon.to_le_bytes())?
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Corrupted taxonomy node information",
            ))?;
        let rank_bytes: [u8; 1] = (*content).try_into().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Corrupted taxonomy node information",
            )
        })?;
        Ok(rank_bytes[0].try_into().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Corrupted taxonomy node information",
            )
        })?)
    }

    pub fn name(&self, taxon: u32) -> std::io::Result<String> {
        let content = self
            .node_ranks
            .get(taxon.to_le_bytes())?
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Corrupted taxonomy node information",
            ))?;
        String::from_utf8((*content).try_into().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Corrupted taxonomy node information",
            )
        })?)
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Corrupted taxonomy node information",
            )
        })
    }

    pub fn query(&self, accession: &str) -> std::io::Result<TaxonomyInfo> {
        let bare_acc = accession.split(".").next().unwrap().as_bytes();

        let taxon_vec = if let Some(node) = self.accession_map.get(&bare_acc)? {
            node
        } else {
            match (
                self.accession_map.get_lt(&bare_acc)?,
                self.accession_map.get_gt(&bare_acc)?,
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
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Corrupted taxonomy node information",
            )
        })?;
        let taxon = u32::from_le_bytes(taxon_bytes.clone());

        let mut ancestor_taxons = vec![];
        let mut ancestor_id = taxon_bytes;
        while ancestor_id != 1u32.to_le_bytes() {
            ancestor_taxons.push(u32::from_le_bytes(ancestor_id));
            if let Ok(Some(content)) = self.node_map.get(ancestor_id) {
                ancestor_id = (*content).try_into().map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Corrupted taxonomy node information",
                    )
                })?
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Corrupted taxonomy node information",
                ));
            }
        }

        let mut result = vec![];

        for taxon in ancestor_taxons {
            result.push((self.rank(taxon)?, self.name(taxon)?));
        }

        Ok(TaxonomyInfo(result))
    }
}