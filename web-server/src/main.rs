mod consts;
mod scrape;
mod storage;

use actix_web::{get, middleware::DefaultHeaders, web, App, HttpResponse, HttpServer, Responder};
use env_logger::Builder;
use log::LevelFilter;
use serde::Deserialize;
use storage::Storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Builder::new()
        .filter_level(LevelFilter::Error)
        .filter_module("web-server", LevelFilter::Debug)
        .init();

    let storage = Storage::new();

    println!("{} anime in storage", storage.cache.len());
    println!("length of top anime: {}", storage.get_top_anime().len());
    println!("length of top airing: {}", storage.get_top_airing().len());

    let storage = web::Data::new(storage);

    //let anime_list = animixplay::parse_anime_list(true).await;
    //let partial_list = animixplay::get_partial_data(anime_list, false).await;
    //
    //let total_length = partial_list.len();
    //
    //for (i, (malid, partial)) in partial_list.into_iter().enumerate() {
    //    println!(
    //        "[{}/{}] Updating variant for {}",
    //        i + 1,
    //        total_length,
    //        malid
    //    );
    //
    //    storage.update_variant(malid, partial).await;
    //}

    HttpServer::new(move || {
        App::new()
            .app_data(storage.clone())
            .service(get_anime_list)
            .service(get_anime_details)
            .wrap(DefaultHeaders::new().add((
                "Access-Control-Allow-Origin",
                "https://clearanime.freemyip.com",
            )))
    })
    .bind("0.0.0.0:8580")?
    .run()
    .await
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum AnimeListKind {
    Airing,
    Top,
}

#[get("/api/get_anime_list/{kind}")]
async fn get_anime_list(
    kind: web::Path<AnimeListKind>,
    storage: web::Data<Storage>,
) -> impl Responder {
    HttpResponse::Ok().json(match kind.as_ref() {
        AnimeListKind::Airing => storage.get_top_airing(),
        AnimeListKind::Top => storage.get_top_anime(),
    })
}

#[get("/api/get_anime")]
async fn get_anime_details(storage: web::Data<Storage>) -> impl Responder {
    HttpResponse::Ok().json(storage.get_all().await)
}
