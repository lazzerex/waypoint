mod app;
mod cli;
mod components;
mod config;
mod domain;
mod infra;
mod services;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1).peekable();

    if args.peek().is_some_and(|arg| arg == "toasty") {
        let database_url = config::Config::load_database_url().unwrap_or_else(|error| {
            eprintln!("configuration error: {error}");
            std::process::exit(1);
        });

        let db = match infra::db::connect(&database_url).await {
            Ok(db) => db,
            Err(_) => {
                eprintln!("database connection failed; check DATABASE_URL");
                std::process::exit(1);
            }
        };

        if let Err(_) = toasty_cli::ToastyCli::new(db).parse_from(args).await {
            eprintln!("toasty command failed");
            std::process::exit(1);
        }
        return;
    }

    if args.peek().is_some_and(|arg| arg == "create-admin") {
        let database_url = config::Config::load_database_url().unwrap_or_else(|error| {
            eprintln!("configuration error: {error}");
            std::process::exit(1);
        });

        let db = match infra::db::connect(&database_url).await {
            Ok(db) => db,
            Err(_) => {
                eprintln!("database connection failed; check DATABASE_URL");
                std::process::exit(1);
            }
        };

        args.next();
        if let Err(error) = cli::create_admin(db, args).await {
            eprintln!("create-admin failed: {error}");
            std::process::exit(1);
        }
        return;
    }

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

    if let Err(error) = topcoat::start(app::router().with(db).build()).await {
        eprintln!("server failed: {error}");
        std::process::exit(1);
    }
}
