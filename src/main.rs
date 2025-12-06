use iocraft::prelude::*;

use crate::components::app::App;

pub mod agent;
pub mod components;
pub mod error;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    print!("{}[2J", 27 as char); // clear console
    element!(App).render_loop().await.unwrap();
}
