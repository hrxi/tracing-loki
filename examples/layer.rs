use std::error::Error;
use std::time::Duration;
use tracing::info;
use tracing::info_span;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;

fn tracing_setup() -> Result<(), Box<dyn Error>> {
    let (layer, task) = tracing_loki::layer(
        Url::parse("http://127.0.0.1:3100").unwrap(),
        vec![("host".into(), "mine".into())].into_iter().collect(),
        vec![].into_iter().collect(),
    )?;
    tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(layer)
        .with(Layer::new())
        .init();
    tokio::spawn(task.start());
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_setup()?;

    info_span!("report", output = "tracing").in_scope(|| {
        info!(
            task = "tracing_setup",
            result = "success",
            "tracing successfully set up"
        );
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}
