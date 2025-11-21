mod backend;
mod cache;
mod config;
mod handler;

use crate::backend::BackendManager;
use crate::handler::serve_image;
use actix_web::{App, HttpServer};
use anyhow::Error;
use clap::Parser;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::info;
use tracing::metadata::LevelFilter;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use tracing_tree::HierarchicalLayer;

#[derive(Parser, Debug)]
#[command(author, version, about = "Image Server", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 启用日志
    let filter = LevelFilter::INFO;
    let tree_layer = HierarchicalLayer::new(2)
        .with_targets(true)
        .with_thread_names(true);
    let subscriber = Registry::default().with(tree_layer).with(filter);
    tracing::subscriber::set_global_default(subscriber)?;

    // 解析命令行参数
    let args = Args::parse();

    // 读取配置
    let cfg_path = Path::new(&args.config);
    if !cfg_path.exists() {
        let template = config::Config::template();
        std::fs::write(cfg_path, template)?;
        tracing::warn!(
            "Config file not found. A default template has been created at {}.\n\
             Please edit the file and rerun the program.",
            cfg_path.display()
        );
        return Err(Error::msg("Please check the configuration file"));
    }
    let cfg = config::Config::from_file(cfg_path)?;
    let backends = BackendManager::new(&cfg.backends).await?;
    let server_config = Arc::new(cfg.server);
    let avif_cfg = Arc::new(cfg.avif);

    // 异步缓存清理任务
    let cache_enabled = server_config.cache_enabled;
    let cache_dir = server_config.cache_dir.clone();
    let ttl = server_config.cache_ttl_secs;
    let max_bytes = server_config.cache_max_bytes();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(ttl));
        loop {
            interval.tick().await;
            cache::clean_cache_dir(&cache_dir, ttl, max_bytes, cache_enabled);
        }
    });

    // 启动 HTTP 服务端
    let server_config_clone = Arc::clone(&server_config);
    info!(
        "Binding server to {}:{}",
        server_config.bind, server_config.port
    );
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(backends.clone()))
            .app_data(actix_web::web::Data::new(server_config_clone.clone()))
            .app_data(actix_web::web::Data::new(avif_cfg.clone()))
            .service(serve_image)
    })
    .bind((server_config.bind.clone(), server_config.port))?
    .run()
    .await?;

    Ok(())
}
