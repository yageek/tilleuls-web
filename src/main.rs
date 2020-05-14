mod crawl_page;
mod models;
mod xlsx;

use log::info;
use warp::Filter;

use handlebars::Handlebars;
#[tokio::main]
async fn main() {
    // Load the templates
    let mut reg = Handlebars::new();
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    info!("Start server on 127.0.0.1:3030...");
    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}
