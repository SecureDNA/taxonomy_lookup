use std::path::PathBuf;

use clap::Parser;
use hex_literal::hex;
use sha2::Digest;

/// Download the taxonomy database
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Where to put the resulting database files. By default, this tool will place them in
    /// "$XDG_DATA_HOME/taxonomy_lookup/", which is where the library expects to find them by
    /// default.
    target: Option<PathBuf>,
}

const GZIP_URL: &str = "https://taxonomylookup.s3.amazonaws.com/taxonomy_db-2022-06-01.tar.gz";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let tmpdir = tempfile::Builder::new()
        .prefix("taxonomy_lookup")
        .tempdir()?;
    let response = reqwest::get(GZIP_URL).await.expect("HTTP Request failed");
    let dest = tmpdir.path().join("taxonomy_lookup.gz");
    let mut f = std::fs::File::create(&dest)?;
    let content = response
        .bytes()
        .await
        .expect("HTTP response could not be read");

    let mut hasher = sha2::Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    assert_eq!(
        result[..],
        hex!("0587d7831f159c4fc1602b3745a1916d3fa0311f39086b9b250e33fd7e85ac52")[..]
    );

    std::io::copy(&mut std::io::Cursor::new(content), &mut f)?;
    taxonomy_lookup::unzip_db(
        &dest,
        args.target
            .unwrap_or(xdg::BaseDirectories::with_prefix("taxonomy_lookup")?.get_data_home()),
    )?;
    Ok(())
}
