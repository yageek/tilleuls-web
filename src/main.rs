mod crawl_page;
mod models;
mod xlsx;

use chrono::{Date, Utc};
use hyper::server::Server;
use listenfd::ListenFd;
use log::info;
use std::convert::Infallible;
use warp::{reject::Reject, Filter};

use crate::models::Item;
use crawl_page::*;
use handlebars::Handlebars;
use models::WeeklyBasketOffer;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;

#[derive(Debug)]
struct AppData {
    offer: Option<WeeklyBasketOffer>,
}

#[derive(Debug)]
struct Order<'a> {
    item: &'a Item,
    quantity: u32,
}

#[derive(Debug)]
enum DataError {
    NotFound,
}

impl Reject for DataError {}

struct Render<'a> {
    hbs: Handlebars<'a>,
}

impl<'a> Default for Render<'a> {
    fn default() -> Self {
        let mut hbs = Handlebars::new();
        hbs.register_template_file("index", "www/templates/index.hbs")
            .unwrap();

        hbs.register_template_file("form", "www/templates/form.hbs")
            .unwrap();
        Render { hbs }
    }
}

impl<'a> Render<'a> {
    fn new() -> Self {
        Render::default()
    }

    fn render<T: Serialize>(&self, template: &str, value: &T) -> impl warp::Reply {
        let value = self
            .hbs
            .render(template, value)
            .unwrap_or_else(|e| e.to_string());
        warp::reply::html(value)
    }
}

#[tokio::main]
async fn main() {
    // Load the templates

    let hbs = Arc::new(Render::default());

    // Data for offers
    let app_data: AppData = AppData { offer: None };
    let app_data_arc = Arc::new(Mutex::new(app_data));
    let with_app_data = warp::any().map(move || app_data_arc.clone());
    let with_render = warp::any().map(move || hbs.clone());

    // Register static files
    let fs = warp::path("static").and(warp::fs::dir("www/static"));

    // Setup communication
    let handle = Handle::current();
    handle.spawn(get_xlsx_data(app_data_arc.clone()));

    // Get /
    let index = warp::path::end()
        .and(with_render)
        .and(with_app_data)
        .map(|r: Arc<Render>, data: Arc<Mutex<AppData>>| r.render::<()>("index", &()));

    // Get /
    // let new_clone = Arc::clone(&app_data_arc);
    // // Order preview
    // let order_preview = warp::path("order")
    //     .and(warp::post())
    //     .and(warp::body::content_length_limit(1024 * 32))
    //     .and(warp::body::form())
    //     .map(move |form: HashMap<String, String>| {
    //         if let Ok(element) = new_clone.lock() {
    //             if let Some(offer) = &element.offer {
    //                 // Retrieve all_elements
    //                 let items: Vec<&Item> = form
    //                     .keys()
    //                     .filter_map(|key| {
    //                         // Items
    //                         if key.starts_with("item_") {
    //                             let indexes: Vec<u32> = key
    //                                 .split("_")
    //                                 .skip(1)
    //                                 .map(|s| s.parse::<u32>().unwrap())
    //                                 .collect();

    //                             Some(
    //                                 &offer.categories()[indexes[0] as usize].items()
    //                                     [indexes[1] as usize],
    //                             )
    //                         } else {
    //                             None
    //                         }
    //                     })
    //                     .collect();

    //                 return format!("Items: {:?}", items);
    //             }
    //         }

    //         return "Hello".to_string();
    //     });

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

async fn get_xlsx_data<'a>(data: Arc<Mutex<AppData>>) {
    info!("Start retrieving xlsx from the server...");

    if let Ok(Some(offer)) = retrieve_new_xlsx(None).await {
        let mut data = data.lock().unwrap();
        data.offer = Some(offer);
    }
}
