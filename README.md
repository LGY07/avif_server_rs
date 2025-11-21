# AVIF Server (Rust)

[中文版本](README_zh-CN.md)

A high-performance image server written in **Rust**, capable of generating **AVIF** and **WebP** images on demand with
caching support. Supports multiple backends including local filesystem, S3, HTTP/UnixSocket, and more via OpenDAL.

## Features

- Serve **AVIF** and **WebP** images
- Resize on-the-fly via query parameters: `?w=width&h=height`
- Configurable caching (TTL and max cache size)
- Multiple backends via OpenDAL:
    - Local filesystem
    - Amazon S3
    - Remote HTTP/UnixSocket
    - Additional backends can be added via OpenDAL
- Multi-threaded encoding using **image crate**
- Simple configuration using TOML

## Build

### Requirements

- Rust (latest stable recommended)
- Cargo build tool

```bash
git clone https://github.com/LGY07/avif_server_rs.git
cd avif_server_rs
cargo build --release
````

## Configuration

Configuration is done via `config.toml`. Example:

```toml
#################################
# Server Configuration
#################################
[server]

# Bind address
bind = "127.0.0.1"

# Port
port = 8080

# Enable caching
cache_enabled = true

# Cache directory
cache_dir = "./cache"

# Cache TTL (seconds, 7 days)
cache_ttl_secs = 604800

# Maximum cache size in MB
cache_max_mb = 1024

# Default format when request has no 'format' parameter
# Accepts "webp" or "avif". Falls back to AVIF if parsing fails
default_format = "avif"

[avif]
quality = 80
speed = 6
thread = 5

#################################
# Backends
#################################
# Multiple backends can be defined using [[backends]] blocks.
# Each backend has a unique URL prefix for routing.
# OpenDAL allows adding many more backend types.

[[backends]]
prefix = "local"
type = "fs"
root = "C:\\Users\\Tuxium\\CLionProjects\\ros-bot\\data\\chat\\image"

# Optional S3 backend
# [[backends]]
# prefix = "s3"
# type = "s3"
# bucket = "my-bucket"
# region = "us-east-1"
# access_key = "ACCESS_KEY"
# secret_key = "SECRET_KEY"
# endpoint = "https://s3.example.com"

# Optional HTTP backend
# [[backends]]
# prefix = "remote"
# type = "http"
# root = "https://example.com/assets"
# unix_socket = "/var/run/custom.sock"
```

### Query Parameters

* `w`: target width
* `h`: target height
* `format`: `avif` or `webp`

Example URL:

```
http://127.0.0.1:8080/local/image.jpg?w=800&h=600&format=webp
```

## Usage

```bash
cargo run --release
```

Or run the compiled binary from `target/release/avif-server-rs`.

## Logging

* Uses `tracing` for structured logging
* Default logs include request info, cache hits/misses, and encoding events

## Cache

* Enabled by default
* Stores generated AVIF/WebP images in `cache_dir`
* Automatically cleans expired files or when total size exceeds limit

## Notes

* `[avif]` settings allow tuning `speed` and `thread` for speed vs quality
* **Supported input formats**: any format supported by Rust `image` crate

## License

MIT License
