use rust_embed::Embed;

#[derive(Embed)]
#[folder = "data/"]
#[include = "*.json"]
pub(crate) struct Asset;
