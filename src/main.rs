// in `src/main.rs`

use axum::{
    Router,
    body::BoxBody,
    http::header,
    response::{IntoResponse, Response},
    routing::get,
};
use reqwest::StatusCode;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{info, warn, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG should be a valid tracing filter");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .finish()
        .with(filter)
        .init();
    let app = Router::new().route("/", get(root_get));

    let addr = "0.0.0.0:5000".parse().unwrap();
    info!("listening on {}", addr);
    
    let quit_sig = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Initiating graceful shutdown");
    };
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(quit_sig)
        .await
        .unwrap();
}

async fn root_get() -> Response<BoxBody> {
    match get_cat_ascii_art().await {
        Ok(art) => (
            StatusCode::OK,
            //           was text/plain 👇
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            art,
        )
            .into_response(),
        Err(e) => {
            println!("Something went wrong: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
        }
    }
}
async fn get_cat_ascii_art() -> color_eyre::Result<String> {
    #[derive(Deserialize)]
    struct CatImage {
        url: String,
    }

    let api_url = "https://api.thecatapi.com/v1/images/search";
    let client = reqwest::Client::default();

    let image = client
        .get(api_url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<CatImage>>()
        .await?
        .pop()
        .ok_or_else(|| color_eyre::eyre::eyre!("The Cat API returned no images"))?;

    let image_bytes = client
        .get(image.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let image = image::load_from_memory(&image_bytes)?;
    let ascii_art = artem::convert(
        image,
        artem::options::OptionBuilder::new()
            .target(artem::options::TargetType::HtmlFile(true, true))
            .build(),
    );

    Ok(ascii_art)
}
