use std::fs;
use actix_files::Files;
use actix_web::{error::ErrorInternalServerError, web, App, HttpResponse, HttpServer};

mod strings;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let addr = "0.0.0.0:80";

    let server = HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(index))
            .service(Files::new("/resource", "resource").show_files_listing())
            .configure(strings::config)
    }).bind(addr)?.run();

    println!("start server");

    server.await
}

// fn main() {
//     strings::test();
// }

async fn index() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().body(
        fs::read_to_string("html/untroche.html")
            .map_err(|_| ErrorInternalServerError("ファイルが見つかりません"))?,
    ))
}
