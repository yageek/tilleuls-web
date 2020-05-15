mod crawl_page;
mod models;
mod xlsx;

use chrono::{Date, Utc};
use hyper::server::Server;
use listenfd::ListenFd;
use log::info;
use std::convert::Infallible;
use warp::Filter;

use models::WeeklyBasketOffer;

use crawl_page::*;
use handlebars::Handlebars;
use std::sync::{Arc, Mutex};

use tokio::runtime::Handle;
use tokio::sync::mpsc;
#[derive(Debug)]
struct ContentData {
    offer: Option<WeeklyBasketOffer>,
}

impl Default for ContentData {
    fn default() -> Self {
        ContentData { offer: None }
    }
}

#[tokio::main]
async fn main() {
    // Load the templates
    let mut reg = Handlebars::new();
    reg.register_template_file("index", "www/templates/form.hbs")
        .unwrap();
    let reg = Arc::new(reg);

    // Register static files
    let fs = warp::path("static").and(warp::fs::dir("www/static"));

    // Data for offers
    let mut offer_data: ContentData = ContentData::default();
    let mut offer_data = Arc::new(Mutex::new(offer_data));

    // Setup communication

    let (tx, mut rx) = mpsc::channel(5);

    let handle = Handle::current();
    handle.spawn(get_xlsx_data(offer_data));

    // Get /
    let index = warp::path::end().map(move || {
        let content = reg
            .render("index", &())
            .unwrap_or_else(|err| err.to_string());

        warp::reply::html(content)
    });

    // Global routes
    let routes = warp::get().and(fs.or(index));

    // Hot reload

    // info!("Start server on 127.0.0.1:3030...");
    // warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    // hyper let's us build a server from a TcpListener (which will be
    // useful shortly). Thus, we'll need to convert our `warp::Filter` into
    // a `hyper::service::MakeService` for use with a `hyper::server::Server`.
    let svc = warp::service(routes);

    let make_svc = hyper::service::make_service_fn(|_: _| {
        // the clone is there because not all warp filters impl Copy
        let svc = svc.clone();
        async move { Ok::<_, Infallible>(svc) }
    });

    let mut listenfd = ListenFd::from_env();
    // if listenfd doesn't take a TcpListener (i.e. we're not running via
    // the command above), we fall back to explicitly binding to a given
    // host:port.
    let server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        Server::from_tcp(l).unwrap()
    } else {
        Server::bind(&([127, 0, 0, 1], 3030).into())
    };

    server.serve(make_svc).await.unwrap();
}

async fn get_xlsx_data(data: Arc<Mutex<ContentData>>) {
    info!("Start retrieving xlsx from the server...");

    if let Ok(Some(offer)) = retrieve_new_xlsx(None).await {
        let mut data = data.lock().unwrap();
        data.offer = Some(offer);
    }
}
