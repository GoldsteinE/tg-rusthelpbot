mod index;
mod scraper;
mod rustdoc_types;

async fn run() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();
    let mut s = scraper::Scraper::new()?;
    let url = s.find_index_url("tokio").await?;
    println!("{}", url);
    let index = s.fetch_index_by_url(url).await?;
    for (crate_name, index) in index {
        let json_path = format!("converted/{}.json", crate_name);
        let converted_index = index::Index::from_rustdoc(index);
        let converted_json = serde_json::to_string_pretty(&converted_index)?;
        std::fs::write(json_path, converted_json)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    run().await
}
