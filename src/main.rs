mod crawl_page;
mod models;
mod xlsx;

use chrono::{Date, Utc};
use hyper::server::Server;
use listenfd::ListenFd;
use log::info;
use std::convert::Infallible;
use warp::Filter;

use crate::models::Item;
use crawl_page::*;
use handlebars::Handlebars;
use models::WeeklyBasketOffer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;

#[derive(Debug)]
struct ContentData {
    offer: Option<WeeklyBasketOffer>,
}
#[derive(Debug)]
struct Order<'a> {
    item: &'a Item,
    quantity: u32,
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
    reg.register_template_file("index", "www/templates/index.hbs")
        .unwrap();

    reg.register_template_file("form", "www/templates/form.hbs")
        .unwrap();

    let reg = Arc::new(reg);

    // Register static files
    let fs = warp::path("static").and(warp::fs::dir("www/static"));

    // Data for offers
    let mut offer_data: ContentData = ContentData::default();
    let mut offer_data = Arc::new(Mutex::new(offer_data));

    // Setup communication

    let handle = Handle::current();
    handle.spawn(get_xlsx_data(Arc::clone(&offer_data)));

    // Get /
    let data_clone = Arc::clone(&offer_data);

    let index = warp::path::end().map(move || {
        // Check the validaty of the template
        if let Ok(element) = data_clone.lock() {
            if let Some(data) = &element.offer {
                let content = reg
                    .render("form", &data)
                    .unwrap_or_else(|err| err.to_string());
                warp::reply::html(content)
            } else {
                let content = reg
                    .render("index", &())
                    .unwrap_or_else(|err| err.to_string());

                warp::reply::html(content)
            }
        } else {
            let content = reg
                .render("index", &())
                .unwrap_or_else(|err| err.to_string());

            warp::reply::html(content)
        }
    });

    // Get /
    let new_clone = Arc::clone(&offer_data);
    // Order preview
    let order_preview = warp::path("order")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .map(move |form: HashMap<String, String>| {
            if let Ok(element) = new_clone.lock() {
                if let Some(offer) = &element.offer {
                    // Retrieve all_elements
                    let items: Vec<&Item> = form
                        .keys()
                        .filter_map(|key| {
                            if key.starts_with("item_") {
                                let indexes: Vec<u32> = key
                                    .split("_")
                                    .skip(1)
                                    .map(|s| s.parse::<u32>().unwrap())
                                    .collect();

                                Some(
                                    &offer.categories()[indexes[0] as usize].items()
                                        [indexes[1] as usize],
                                )
                            } else {
                                None
                            }
                        })
                        .collect();

                    return format!("Items: {:?}", items);
                }
            }

            return "Hello".to_string();
        });

    // Global routes
    let routes = warp::get().and(fs.or(index)).or(order_preview);

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
