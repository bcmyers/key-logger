use std::sync::{Arc, Mutex};

use actix_web::web::Data;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use failure::Error;

use crate::key_logger::{Key, State};

const HOST: &str = "127.0.0.1";

pub fn run_server(port: u16, state: Arc<Mutex<State<Key>>>) -> Result<(), Error> {
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(web::resource("/").route(web::get().to_async(index)))
    })
    .bind(format!("{}:{}", HOST, port))?
    .run()
    .map_err(Error::from)
}

fn index(state: Data<Arc<Mutex<State<Key>>>>, _req: HttpRequest) -> HttpResponse {
    let guard = state.lock().unwrap();
    HttpResponse::Ok().body(format!("{:x?}", guard.data()))
}
