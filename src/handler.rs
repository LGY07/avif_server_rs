use crate::backend::BackendManager;
use crate::cache;
use crate::config::{AvifConfig, ServerConfig};
use actix_web::{HttpResponse, Responder, get, web};
use image::ExtendedColorType;
use rgb::bytemuck::cast_slice;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

#[get("/{prefix}/{tail:.*}")]
pub async fn serve_image(
    path: web::Path<(String, String)>,
    query: web::Query<std::collections::HashMap<String, String>>,
    backends: web::Data<BackendManager>,
    server_config: web::Data<Arc<ServerConfig>>,
    avif_config: web::Data<Arc<AvifConfig>>,
) -> impl Responder {
    // 解析 URL
    let (prefix, tail) = path.into_inner();
    let width: u32 = query.get("w").and_then(|s| s.parse().ok()).unwrap_or(0);
    let height: u32 = query.get("h").and_then(|s| s.parse().ok()).unwrap_or(0);
    let format = query
        .get("format")
        .map(|s| s.to_owned())
        .unwrap_or("avif".to_string());

    // 获取原图
    let backend = match backends.get(&prefix) {
        Some(b) => b,
        None => return HttpResponse::BadRequest().body("Unknown backend prefix"),
    };
    let orig_data = match backend.read(&tail).await {
        Ok(d) => d,
        Err(_) => return HttpResponse::NotFound().body("Original image not found"),
    }
    .to_vec();

    // 生成缓存 key & 文件路径
    let key = cache::cache_key(&orig_data, width, height, &*format);
    let cache_file =
        cache::cache_file_path(&server_config.cache_dir, &key, width, height, &*format);
    info!("Get {}", key);

    // 缓存命中
    if server_config.cache_enabled && cache_file.exists() {
        let content_type = match format.as_str() {
            "avif" => "image/avif",
            "webp" => "image/webp",
            _ => "application/octet-stream",
        };
        info!("Cache hit: {}", key);
        return HttpResponse::Ok()
            .content_type(content_type)
            .body(fs::read(&cache_file).unwrap_or(orig_data));
    }

    // Content-Type
    let content_type = match format.as_str() {
        "avif" => "image/avif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };

    // 编码 AVIF/WebP
    let processed = web::block(move || -> anyhow::Result<Vec<u8>> {
        // 使用 image crate 解码
        let mut img = image::ImageReader::new(std::io::Cursor::new(&orig_data))
            .with_guessed_format()?
            .decode()?;

        // resize
        if width > 0 || height > 0 {
            let target_width = if width > 0 { width } else { img.width() };
            let target_height = if height > 0 { height } else { img.height() };
            img = img.resize_exact(
                target_width,
                target_height,
                image::imageops::FilterType::CatmullRom,
            );
        }

        // 编码
        let mut out_bytes = Vec::new();
        match format.as_str() {
            "webp" => {
                image::codecs::webp::WebPEncoder::new_lossless(&mut out_bytes).encode(
                    &img.to_rgba8(),
                    img.width(),
                    img.height(),
                    ExtendedColorType::Rgba8,
                )?;
            }
            "avif" => {
                out_bytes = ravif::Encoder::new()
                    .with_num_threads(avif_config.thread)
                    .with_quality(avif_config.quality as f32)
                    .with_speed(avif_config.speed)
                    .encode_rgba(ravif::Img::new(
                        cast_slice(img.to_rgba8().as_raw()),
                        img.width() as usize,
                        img.height() as usize,
                    ))?
                    .avif_file;
            }
            _ => match &*server_config.default_format {
                "webp" => {
                    image::codecs::webp::WebPEncoder::new_lossless(&mut out_bytes).encode(
                        &img.to_rgba8(),
                        img.width(),
                        img.height(),
                        ExtendedColorType::Rgba8,
                    )?;
                }
                "avif" => {
                    out_bytes = ravif::Encoder::new()
                        .with_num_threads(avif_config.thread)
                        .with_quality(avif_config.quality as f32)
                        .with_speed(avif_config.speed)
                        .encode_rgba(ravif::Img::new(
                            cast_slice(img.to_rgba8().as_raw()),
                            img.width() as usize,
                            img.height() as usize,
                        ))?
                        .avif_file;
                }
                _ => {
                    warn!("The default format is not set. Use AVIF");
                    out_bytes = ravif::Encoder::new()
                        .with_num_threads(avif_config.thread)
                        .with_quality(avif_config.quality as f32)
                        .with_speed(avif_config.speed)
                        .encode_rgba(ravif::Img::new(
                            cast_slice(img.to_rgba8().as_raw()),
                            img.width() as usize,
                            img.height() as usize,
                        ))?
                        .avif_file;
                }
            },
        }

        // 写缓存
        if server_config.cache_enabled {
            if let Some(parent) = Path::new(&cache_file).parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&cache_file, &out_bytes)?;
        }
        info!("Encoding completed: {}", key);

        Ok(out_bytes)
    })
    .await
    .expect("Encoding failure")
    .expect("Encoding failure");

    HttpResponse::Ok()
        .content_type(content_type)
        .body(processed)
}
