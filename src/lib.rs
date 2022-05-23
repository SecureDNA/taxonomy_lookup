use std::str::FromStr;

/// Geez NCBI, make up your mind.
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Rank {
    NoRank = 0,

    Clade = 1,

    Superkingdom = 2,
    Kingdom = 3,
    Subkingdom = 4,

    Superphylum = 5,
    Phylum = 6,
    Subphylum = 7,

    Superclass = 8,
    Class = 9,
    Subclass = 10,
    Infraclass = 11,

    Cohort = 12,
    Subcohort = 13,

    Superorder = 14,
    Order = 15,
    Suborder = 16,
    Infraorder = 17,
    Parvorder = 18,

    Superfamily = 19,
    Family = 20,
    Subfamily = 21,

    Tribe = 22,
    Subtribe = 23,

    Genus = 24,
    Subgenus = 25,

    Section = 26,
    Subsection = 27,
    Series = 28,

    SpeciesGroup = 29,
    SpeciesSubgroup = 30,
    Species = 31,
    Subspecies = 32,

    Morph = 33,
    Varietas = 34,
    Forma = 35,
    FormaSpecialis = 36,
    Pathogroup = 37,
    Strain = 38,
    Serogroup = 39,
    Serotype = 40,
    Genotype = 41,
    Biotype = 42,
    Isolate = 43,
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
