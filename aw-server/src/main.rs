#[macro_use]
extern crate log;

use std::env;
use std::path::PathBuf;

use clap::crate_version;
use clap::Parser;

use aw_server::*;

#[cfg(target_os = "linux")]
use sd_notify::NotifyState;
#[cfg(all(target_os = "linux", target_arch = "x86"))]
extern crate jemallocator;
#[cfg(all(target_os = "linux", target_arch = "x86"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Rust server for ActivityWatch
#[derive(Parser)]
#[clap(version = crate_version!(), author = "Johan Bjäreholt, Erik Bjäreholt, et al.")]
struct Opts {
    /// Run in testing mode
    #[clap(long)]
    testing: bool,

    /// Verbose output
    #[clap(long)]
    verbose: bool,

    /// Address to listen to
    #[clap(long)]
    host: Option<String>,

    /// Port to listen on
    #[clap(long)]
    port: Option<String>,

    /// PostgreSQL database host
    #[clap(long)]
    db_host: Option<String>,

    /// PostgreSQL database port
    #[clap(long)]
    db_port: Option<u16>,

    /// PostgreSQL database user
    #[clap(long)]
    db_user: Option<String>,

    /// PostgreSQL database password
    #[clap(long)]
    db_password: Option<String>,

    /// PostgreSQL database name
    #[clap(long)]
    db_name: Option<String>,

    /// Path to webui override
    #[clap(long)]
    webpath: Option<String>,

    /// Mapping of custom static paths to serve, in the format: watcher1=/path,watcher2=/path2
    #[clap(long)]
    custom_static: Option<String>,

    /// Device ID override
    #[clap(long)]
    device_id: Option<String>,

    /// Don't import from aw-server-python if no aw-server-rust db found
    /// (Note: Legacy import not supported with PostgreSQL)
    #[clap(long)]
    no_legacy_import: bool,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let opts: Opts = Opts::parse();

    use std::sync::Mutex;

    let mut testing = opts.testing;

    // Always override environment if --testing is specified
    if !testing && cfg!(debug_assertions) {
        testing = true;
    }

    logging::setup_logger("aw-server-rust", testing, opts.verbose)
        .expect("Failed to setup logging");

    if testing {
        info!("Running server in Testing mode");
    }

    let mut config = config::create_config(testing);

    // set host if overridden
    if let Some(host) = opts.host {
        config.address = host;
    }

    // set port if overridden
    if let Some(port) = opts.port {
        config.port = port.parse().unwrap();
    }

    // set custom_static if overridden, transform into map
    if let Some(custom_static_str) = opts.custom_static {
        let custom_static_map: std::collections::HashMap<String, String> = custom_static_str
            .split(',')
            .map(|s| {
                let mut split = s.split('=');
                let key = split.next().unwrap().to_string();
                let value = split.next().unwrap().to_string();
                (key, value)
            })
            .collect();
        config.custom_static.extend(custom_static_map);

        // validate paths, log error if invalid
        // remove invalid paths
        for (name, path) in config.custom_static.clone().iter() {
            if !std::path::Path::new(path).exists() {
                error!("custom_static path for {} does not exist ({})", name, path);
                config.custom_static.remove(name);
            }
        }
    }

    // Configure PostgreSQL database connection
    let mut db_config = aw_datastore::DbConfig::from_env();
    
    // Override with CLI arguments if provided
    if let Some(host) = opts.db_host {
        db_config.host = host;
    }
    if let Some(port) = opts.db_port {
        db_config.port = port;
    }
    if let Some(user) = opts.db_user {
        db_config.user = user;
    }
    if let Some(password) = opts.db_password {
        db_config.password = password;
    }
    if let Some(name) = opts.db_name {
        db_config.database = name;
    }
    
    info!(
        "Using PostgreSQL database at {}:{}/{}",
        db_config.host, db_config.port, db_config.database
    );

    let asset_path = opts.webpath.map(|webpath| PathBuf::from(webpath));
    info!("Using aw-webui assets at path {:?}", asset_path);

    // Note: Legacy import from aw-server-python not supported with PostgreSQL
    let legacy_import = false;
    if !opts.no_legacy_import {
        warn!("Legacy import from aw-server-python is not supported with PostgreSQL backend");
    }

    let device_id: String = if let Some(id) = opts.device_id {
        id
    } else {
        device_id::get_device_id()
    };

    // Create datastore (async operation)
    let datastore = aw_datastore::Datastore::new_with_config(db_config, legacy_import)
        .await
        .expect("Failed to initialize PostgreSQL datastore");

    let server_state = endpoints::ServerState {
        // PostgreSQL backend - legacy import not supported
        datastore: Mutex::new(datastore),
        asset_resolver: endpoints::AssetResolver::new(asset_path),
        device_id,
    };

    let _rocket = endpoints::build_rocket(server_state, config)
        .ignite()
        .await?;
    #[cfg(target_os = "linux")]
    let _ = sd_notify::notify(true, &[NotifyState::Ready]);
    _rocket.launch().await?;

    Ok(())
}
