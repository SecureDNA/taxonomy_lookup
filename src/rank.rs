use std::str::FromStr;

/// Geez NCBI, make up your mind.
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
