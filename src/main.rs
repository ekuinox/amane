#[macro_use]
extern crate anyhow;

mod http;
mod bucket;
mod state;

use actix_web::{App, HttpServer};
use http::routes::*;
use state::AppState;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let matches = clap::App::new(clap::crate_name!())
        .args(&[
            clap::Arg::with_name("bind")
                .takes_value(true)
                .short("b"),
            clap::Arg::with_name("directory")
                .takes_value(true)
                .short("d")
        ])
        .get_matches();
    let state = AppState {
        data_directory: matches
            .value_of("directory")
            .unwrap_or("./tmp")
            .to_string(),
    };

    let server = HttpServer::new(move || App::new()
        .data(state.clone())
        .service(search_files)
        .service(put_file)
        .service(get_file)
        .service(delete_file)
    );

    server
        .bind(matches
            .value_of("bind")
            .unwrap_or("0.0.0.0:8080")
        )?
        .run()
        .await?;

    Ok(())
}
