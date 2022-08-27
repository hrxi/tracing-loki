tracing-loki
============

A [tracing](https://github.com/tokio-rs/tracing) layer for [Grafana
Loki](https://grafana.com/oss/loki/).

[![Build status](https://github.com/hrxi/tracing-loki/actions/workflows/build.yaml/badge.svg)](https://github.com/hrxi/tracing-loki/actions/workflows/build.yaml)

Documentation
-------------

https://docs.rs/tracing-loki

Usage
-----

Add this to your `Cargo.toml`:
```toml
[dependencies]
tracing-loki = "0.1"
```

Example
-------

```rust
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), tracing_loki::Error> {
    let (layer, task) = tracing_loki::layer(
        Url::parse("http://127.0.0.1:3100").unwrap(),
        vec![("host".into(), "mine".into())].into_iter().collect(),
        vec![].into_iter().collect(),
        None   // Tenant id that is passed as X-Scope-OrgID Header value to support multi-tenancy if enabled
    )?;

    // We need to register our layer with `tracing`.
    tracing_subscriber::registry()
        .with(layer)
        // One could add more layers here, for example logging to stdout:
        // .with(tracing_subscriber::fmt::Layer::new())
        .init();

    // The background task needs to be spawned so the logs actually get
    // delivered.
    tokio::spawn(task);

    tracing::info!(
        task = "tracing_setup",
        result = "success",
        "tracing successfully set up",
    );

    Ok(())
}
```
