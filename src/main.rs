mod crawl_page;
mod models;
mod xlsx;

use chrono::{Date, Utc};
use hyper::server::Server;
use listenfd::ListenFd;
use log::info;
use std::convert::Infallible;
use warp::{reject::Reject, Filter};

use crate::models::*;
use crawl_page::*;
use handlebars::Handlebars;
use models::WeeklyBasketOffer;

use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::runtime::Handle;

#[derive(Debug)]
struct AppData<'a> {
    offer: Option<WeeklyBasketOffer>,
    sessions: SessionRegistry<'a>,
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

        hbs.register_template_file("make_order", "www/templates/make_order.hbs")
            .unwrap();

        hbs.register_template_file("order_preview", "www/templates/order_preview.hbs")
            .unwrap();
        Render { hbs }
    }
}

impl<'a> Render<'a> {
    fn render<T: Serialize>(&self, template: &str, value: Option<&T>) -> String {
        if let Some(content) = value {
            self.hbs
                .render(template, &content)
                .unwrap_or_else(|e| e.to_string())
        } else {
            self.hbs
                .render(template, &())
                .unwrap_or_else(|e| e.to_string())
        }
    }

    fn render_html<T: Serialize>(&self, template: &str, value: Option<&T>) -> impl warp::Reply {
        let output = self.render(template, value);
        warp::reply::html(output)
    }
}

fn render_order_preview<'a>(
    app_data: &'a AppData,
    form: HashMap<String, String>,
) -> Vec<OrderItem<'a>> {
    if let Some(offer) = &app_data.offer {
        // Retrieve all_elements
        let orders: Vec<OrderItem<'_>> = form
            .iter()
            .filter_map(|(key, value)| {
                // Items
                if key.starts_with("item_") && value != "0" {
                    let indexes: Vec<u32> = key
                        .split("_")
                        .skip(1)
                        .map(|s| s.parse::<u32>().unwrap())
                        .collect();

                    let item =
                        &offer.categories()[indexes[0] as usize].items()[indexes[1] as usize];
                    let quantity = value.parse::<u32>().unwrap();

                    Some(OrderItem::new(item, quantity))
                } else {
                    None
                }
            })
            .collect();
        return orders;
    }
    return vec![];
}

#[tokio::main]
async fn main() {
    // Load the data
    let app_data_arc = Arc::new(RwLock::new(AppData {
        offer: None,
        sessions: SessionRegistry::new(),
    }));

    // Templates
    let hbs_arc = Arc::new(Render::default());
    let hbs_filter = warp::any().map(move || hbs_arc.clone());
    let hbs = move || hbs_filter.clone();

    // XLSX retrieval
    let handle = Handle::current();
    handle.spawn(get_xlsx_data(app_data_arc.clone()));

    // Register static files
    let fs = warp::path("static").and(warp::fs::dir("www/static"));

    // Setup communication
    // Get /

    // Load the session
    let app_data = app_data_arc.clone();
    let index = warp::path::end().and(hbs()).map(move |hbs: Arc<Render>| {
        let data = app_data.read().unwrap();

        if let Some(offer) = &data.offer {
            hbs.render_html("make_order", Some(offer))
        } else {
            hbs.render_html("index", None)
        }
    });

    // Get /
    let app_data = app_data_arc.clone();
    // let hbs = hbs_arc.clone();

    let make_order = warp::path("order")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .and(hbs())
        .map(move |form: HashMap<String, String>, hbs: Arc<Render>| {
            let app_data_x = app_data.write().unwrap();

            let items = render_order_preview(&app_data_x, form);

            let order_preview = Cart::new(items);
            // let string = hbs.render("order_preview", Some(&order_preview));

            let session = Session {
                cart: Some(order_preview),
            };

            let key = SessionRegistry::random_key(48);
            // app_data_x.sessions.insert_session(key, session);
            hbs.render_html::<()>("index", None)
        });
    // Global routes
    let routes = warp::get().and(fs.or(index)); //.or(make_order);

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

async fn get_xlsx_data<'a>(data: Arc<RwLock<AppData<'a>>>) {
    info!("Start retrieving xlsx from the server...");

    if let Ok(Some(offer)) = retrieve_new_xlsx(None).await {
        let mut data = data.write().unwrap();
        data.offer = Some(offer);
    }
}
