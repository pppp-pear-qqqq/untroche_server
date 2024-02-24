use std::fs;
use actix_files::Files;
use actix_web::{error::ErrorInternalServerError, web, App, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let addr = "0.0.0.0:80";

    let server = HttpServer::new(move || {
        App::new()
            .service(Files::new("/css", "resource/css/").show_files_listing())
            .route("/", web::get().to(index))
            .configure(strings_config)
    }).bind(addr)?.run();

    println!("start server");

    server.await
}

fn strings_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/strings")
        .route("/", web::get().to(index))
    );
}

async fn index() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().body(
        fs::read_to_string("html/untroche.html")
            .map_err(|_| ErrorInternalServerError("ファイルが見つかりません"))?,
    ))
}
