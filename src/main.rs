use clap::Parser;
use iocraft::prelude::*;

use crate::components::app::App;

pub mod agent;
pub mod commands;
pub mod components;
pub mod error;
pub mod util;
#[derive(clap::Parser)]
struct Cli {}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let _args = Cli::parse();
    print!("{}[2J", 27 as char); // clear console
    element!(App).render_loop().await.unwrap();
}
