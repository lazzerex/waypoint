mod app;
mod components;
mod config;
mod domain;
mod infra;
mod services;

#[tokio::main]
async fn main() {
    let config = config::Config::load().unwrap_or_else(|error| {
        eprintln!("configuration error: {error}");
        std::process::exit(1);
    });

    let db = match infra::db::connect(&config.database_url).await {
        Ok(db) => db,
        Err(_) => {
            eprintln!("database connection failed; check DATABASE_URL");
            std::process::exit(1);
        }
    };

    let mut args = std::env::args().skip(1).peekable();
    if args.peek().is_some_and(|arg| arg == "toasty") {
        if let Err(error) = toasty_cli::ToastyCli::new(db).parse_from(args).await {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
        return;
    }

    if let Err(error) = topcoat::start(app::router().with(db).build()).await {
        eprintln!("server failed: {error}");
        std::process::exit(1);
    }
}
