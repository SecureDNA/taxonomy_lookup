use std::str::FromStr;

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Geez NCBI, make up your mind.
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
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
    type Err = (); // TODO: when GATs are stablilized, this can be `type Err<'a> = &'a str` so we can hand back the input

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

impl From<Rank> for &'static str {
    fn from(rank: Rank) -> Self {
        match rank {
            Rank::NoRank => "no rank",
            Rank::Clade => "clade",
            Rank::Superkingdom => "superkingdom",
            Rank::Kingdom => "kingdom",
            Rank::Subkingdom => "subkingdom",
            Rank::Superphylum => "superphylum",
            Rank::Phylum => "phylum",
            Rank::Subphylum => "subphylum",
            Rank::Superclass => "superclass",
            Rank::Class => "class",
            Rank::Subclass => "subclass",
            Rank::Infraclass => "infraclass",
            Rank::Cohort => "cohort",
            Rank::Subcohort => "subcohort",
            Rank::Superorder => "superorder",
            Rank::Order => "order",
            Rank::Suborder => "suborder",
            Rank::Infraorder => "infraorder",
            Rank::Parvorder => "parvorder",
            Rank::Superfamily => "superfamily",
            Rank::Family => "family",
            Rank::Subfamily => "subfamily",
            Rank::Tribe => "tribe",
            Rank::Subtribe => "subtribe",
            Rank::Genus => "genus",
            Rank::Subgenus => "subgenus",
            Rank::Section => "section",
            Rank::Subsection => "subsection",
            Rank::Series => "series",
            Rank::SpeciesGroup => "species group",
            Rank::SpeciesSubgroup => "species subgroup",
            Rank::Species => "species",
            Rank::Subspecies => "subspecies",
            Rank::Morph => "morph",
            Rank::Varietas => "varietas",
            Rank::Forma => "forma",
            Rank::FormaSpecialis => "forma specialis",
            Rank::Pathogroup => "pathogroup",
            Rank::Strain => "strain",
            Rank::Serogroup => "serogroup",
            Rank::Serotype => "serotype",
            Rank::Genotype => "genotype",
            Rank::Biotype => "biotype",
            Rank::Isolate => "isolate",
        }
    }
}
