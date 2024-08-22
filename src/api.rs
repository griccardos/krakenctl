use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse, Responder,
};

use crate::AppState;

#[get("/")]
pub async fn root() -> impl Responder {
    HttpResponse::Ok().body("Use paths:cpugpu, blank, liquid")
}

#[get("/values/{string}")]
pub async fn values(string: Path<String>, appstate: Data<AppState>) -> impl Responder {
    let mut manager = appstate.manager.lock().unwrap();
    manager.set_values_from_input(&string,false);
    HttpResponse::Ok()
}

pub async fn details(data: Data<AppState>) -> String {
    let manager = data.manager.lock().unwrap();
    format!("Request details: {}", manager.details())
}
