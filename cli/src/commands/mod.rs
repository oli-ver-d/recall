use crate::{
    api::ApiClient,
    cli::{Args, Commands},
    display,
};
use std::error::Error;

pub mod open;
pub mod save;
pub mod search;

pub async fn execute(args: Args) -> Result<(), Box<dyn Error>> {
    let client = ApiClient::new(args.server.clone());

    match args.command {
        Commands::Search {
            query,
            limit,
            tags,
            whole,
            title,
        } => {
            search::handle_search(&client, &args.server, &query, limit, &tags, whole, title).await?;
        }
        Commands::Save { url, tags } => {
            save::handle_save(&client, &url, tags).await?;
        }
        Commands::Open { id, original } => {
            open::handle_open(&client, id, original).await?;
        }
    }

    Ok(())
}
