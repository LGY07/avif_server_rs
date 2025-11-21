# AVIF Server (Rust)

[English version](README.md)

一个使用 **Rust** 编写的高性能图片服务器，可按需生成 **AVIF** 和 **WebP**
图片，并支持缓存。支持多种后端，包括本地文件系统、S3、HTTP/UnixSocket，并且可以通过 OpenDAL 添加更多后端。

## 功能

- 提供 **AVIF** 和 **WebP** 图片服务
- 支持通过查询参数自动调整大小：`?w=宽度&h=高度`
- 可配置缓存（TTL 和最大缓存大小）
- 多后端支持（通过 OpenDAL）：
    - 本地文件系统
    - Amazon S3
    - 远程 HTTP/UnixSocket
    - 可通过 OpenDAL 添加更多后端
- 高效多线程编码，使用 **`image` crate**
- 通过 **TOML 配置文件**轻松配置

## 构建

### 前置条件

- Rust（推荐最新稳定版）
- `cargo` 构建工具

```bash
git clone https://github.com/LGY07/avif_server_rs.git
cd avif_server_rs
cargo build --release
````

## 配置

配置通过 `config.toml` 完成。示例：

```toml
#################################
# 服务器配置
#################################
[server]

# 绑定地址
bind = "127.0.0.1"

# 端口
port = 8080

# 启用缓存
cache_enabled = true

# 缓存目录
cache_dir = "./cache"

# 缓存 TTL（秒，7 天）
cache_ttl_secs = 604800

# 最大缓存大小（MB）
cache_max_mb = 1024

# 当请求未指定 format 参数时的默认输出格式
# 可选 "webp" 或 "avif"，解析失败则使用 AVIF
default_format = "avif"

[avif]
# AVIF 编码质量（0-100）
quality = 80
# 编码速度与质量的折中 (0=慢/最佳, 10=快/低质量)
speed = 6
# 编码线程数
thread = 5

#################################
# 后端配置
#################################
# 可使用 [[backends]] 块定义多个后端，每个后端有唯一的 URL 前缀
# 通过 OpenDAL 可添加更多后端类型

[[backends]]
prefix = "local"
type = "fs"
root = "C:\\Users\\Tuxium\\CLionProjects\\ros-bot\\data\\chat\\image"

# 可选 S3 后端
# [[backends]]
# prefix = "s3"
# type = "s3"
# bucket = "my-bucket"
# region = "us-east-1"
# access_key = "ACCESS_KEY"
# secret_key = "SECRET_KEY"
# endpoint = "https://s3.example.com"

# 可选 HTTP 后端
# [[backends]]
# prefix = "remote"
# type = "http"
# root = "https://example.com/assets"
# unix_socket = "/var/run/custom.sock"
```

### 查询参数

* `w`：目标宽度
* `h`：目标高度
* `format`：`avif` 或 `webp`

示例 URL：

```
http://127.0.0.1:8080/local/image.jpg?w=800&h=600&format=webp
```

## 使用

```bash
cargo run --release
```

或直接运行编译后的二进制文件 `target/release/avif-server-rs`。

## 日志

* 使用 `tracing` 记录结构化日志
* 默认日志包括请求信息、缓存命中/未命中、编码事件

## 缓存

* 默认启用
* 存储生成的 AVIF/WebP 图片在 `cache_dir`
* 自动清理超过 TTL 或超过最大缓存大小的文件

## 注意事项

* 可通过 `[avif]` 配置的 `speed` 和 `thread` 调整速度与质量的平衡
* **支持格式**：输入图片支持 Rust `image` crate 支持的任意格式

## 许可证

MIT 许可证

