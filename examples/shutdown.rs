use std::error::Error;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;
use tracing::info_span;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;

struct LokiTask {
    // shutdown handler, used to shutdown io task
    shutdown: tracing_loki::LayerShutdownHandler,
    // signal io task gracefully quit to `main`
    task_await: Arc<Notify>,
}

impl LokiTask {
    async fn shutdown_and_wait(self) {
        // notify io task to shutdown
        // io task will quit after sent all queued messages
        self.shutdown.shutdown();

        // wait task quit, add `timeout` if needed
        self.task_await.notified().await;
    }
}

fn tracing_setup() -> Result<LokiTask, Box<dyn Error>> {
    let (layer, task) = tracing_loki::layer(
        Url::parse("http://127.0.0.1:3100").unwrap(),
        vec![("host".into(), "mine".into())].into_iter().collect(),
        vec![].into_iter().collect(),
    )?;

    let loki_task = LokiTask {
        shutdown: layer.create_shutdown_handler(),
        task_await: Arc::new(Notify::new()),
    };

    tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(layer)
        .with(Layer::new())
        .init();

    // spawn io task
    {
        let notify = Arc::clone(&loki_task.task_await);
        tokio::spawn(async move {
            // actual io task
            task.await;

            // notify io task has been done
            notify.notify_one();
        });
    }

    Ok(loki_task)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let task = tracing_setup()?;

    info_span!("report", output = "tracing").in_scope(|| {
        info!(
            task = "tracing_setup",
            result = "success",
            "tracing successfully set up"
        );
    });

    task.shutdown_and_wait().await;

    Ok(())
}
