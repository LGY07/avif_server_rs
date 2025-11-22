use blake3;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// 生成缓存 key：
/// 用原图 bytes + 尺寸 + 格式进行 blake3 哈希
pub fn cache_key(original_data: &[u8], width: u32, height: u32, format: &str) -> String {
    debug!(
        "Generating cache key for {} bytes {}x{} {}",
        original_data.len(),
        width,
        height,
        format
    );

    let mut hasher = blake3::Hasher::new();
    hasher.update(original_data);
    hasher.update(format!("{}x{}:{}", width, height, format).as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// 根据 cache key 生成文件路径：
/// 采用前两位作为子目录，减少单目录文件数。
pub fn cache_file_path(
    cache_dir: &str,
    key: &str,
    width: u32,
    height: u32,
    format: &str,
) -> PathBuf {
    let subdir = &key[..2];

    // 缓存文件名：hash_widthxheight.format
    // 如：ab/ab1234cd56_800x600.avif
    let path = Path::new(cache_dir)
        .join(subdir)
        .join(format!("{}_{}x{}.{}", key, width, height, format));

    // 创建目录，ignore 错误
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }

    debug!("Resolved cache file path: {:?}", path);

    path
}

/// 删除超过 TTL 的文件
/// 如果总大小超过上限，按时间从旧到新删除
pub fn clean_cache_dir(cache_dir: &str, ttl_secs: u64, max_total_bytes: u64, cache_enabled: bool) {
    if !cache_enabled {
        debug!("Cache disabled, skipping cleanup");
        return;
    }

    info!("Starting cache cleanup…");

    let mut total_size = 0u64;
    let mut entries: Vec<(PathBuf, u64, SystemTime)> = Vec::new();

    // 遍历二级目录
    if let Ok(subdirs) = fs::read_dir(cache_dir) {
        for subdir in subdirs.flatten() {
            if let Ok(files) = fs::read_dir(subdir.path()) {
                for file in files.flatten() {
                    // 读取文件元信息
                    if let Ok(meta) = file.metadata() {
                        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        let size = meta.len();
                        total_size += size;

                        entries.push((file.path(), size, modified));
                    }
                }
            }
        }
    }

    let before_size = total_size;
    let now = SystemTime::now();

    // 删除过期文件
    for (path, size, modified) in &entries {
        if now.duration_since(*modified).unwrap_or_default().as_secs() > ttl_secs {
            warn!("Removing expired cache file: {:?}", path);
            let _ = fs::remove_file(path);
            total_size = total_size.saturating_sub(*size);
        }
    }

    // 超过最大缓存大小时，按最旧删除
    if total_size > max_total_bytes {
        warn!(
            "Cache size {}MB exceeds limit {}MB, cleaning...",
            total_size / 1024 / 1024,
            max_total_bytes / 1024 / 1024
        );

        // 时间从旧到新排序
        entries.sort_by_key(|(_, _, modified)| *modified);

        for (path, size, _) in &entries {
            if total_size <= max_total_bytes {
                break;
            }
            warn!("Removing file due to size limit: {:?}", path);
            let _ = fs::remove_file(path);
            total_size = total_size.saturating_sub(*size);
        }
    }

    info!(
        "Cache cleanup finished. Before: {}MB → After: {}MB",
        before_size / 1024 / 1024,
        total_size / 1024 / 1024
    );
}
