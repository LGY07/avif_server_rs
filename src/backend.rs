use crate::config::{BackendConfig, BackendKind};
use opendal::Operator;
use opendal::layers::HttpClientLayer;
use opendal::services;
use std::collections::HashMap;

#[derive(Clone)]
pub struct BackendManager {
    pub backends: HashMap<String, Operator>,
}

impl BackendManager {
    pub async fn new(configs: &[BackendConfig]) -> anyhow::Result<Self> {
        let mut map = HashMap::new();
        for b in configs {
            let op = match &b.kind {
                BackendKind::FileSystem { root } => {
                    Operator::new(services::Fs::default().root(root))?.finish()
                }
                BackendKind::S3 {
                    bucket,
                    region,
                    access_key,
                    secret_key,
                    endpoint,
                } => {
                    let mut builder = services::S3::default()
                        .bucket(bucket)
                        .region(region)
                        .access_key_id(access_key)
                        .secret_access_key(secret_key);
                    if let Some(ep) = endpoint {
                        builder = builder.endpoint(ep);
                    }
                    Operator::new(builder)?.finish()
                }
                BackendKind::Http { root, unix_socket } => {
                    if let Some(sock_path) = unix_socket {
                        #[cfg(target_family = "unix")]
                        {
                            use reqwest::Client;
                            let client = Client::builder()
                                .connect_timeout(std::time::Duration::from_secs(10))
                                .unix_socket(sock_path.to_owned())
                                .build()?;
                            let op_client = opendal::raw::HttpClient::with(client);
                            Operator::new(services::Http::default().root(root))?
                                .finish()
                                .layer(HttpClientLayer::new(op_client))
                        }

                        #[cfg(not(target_family = "unix"))]
                        panic!("UnixSocket is not supported on this platforms");
                    } else {
                        Operator::new(services::Http::default().root(root))?.finish()
                    }
                }
            };
            map.insert(b.prefix.clone(), op);
        }
        Ok(Self { backends: map })
    }

    pub fn get(&self, prefix: &str) -> Option<&Operator> {
        self.backends.get(prefix)
    }
}
