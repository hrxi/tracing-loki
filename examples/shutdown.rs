use std::error::Error;
use tokio::task::JoinHandle;
use tracing::info;
use tracing::info_span;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;

fn tracing_setup(
) -> Result<(tracing_loki::BackgroundTaskController, JoinHandle<()>), Box<dyn Error>> {
    let (layer, controller, task) = tracing_loki::builder()
        .label("host", "mine")?
        .build_controller_url(Url::parse("http://127.0.0.1:3100").unwrap())?;

    tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(layer)
        .with(Layer::new())
        .init();
    Ok((controller, tokio::spawn(task.start())))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let (controller, task) = tracing_setup()?;

    info_span!("report", output = "tracing").in_scope(|| {
        info!(
            task = "tracing_setup",
            result = "success",
            "tracing successfully set up"
        );
    });

    controller.shutdown().await;
    task.await.unwrap();

    Ok(())
}
