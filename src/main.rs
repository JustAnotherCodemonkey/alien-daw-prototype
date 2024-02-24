use tokio::runtime::Builder;
use tracing::instrument;

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();
    
    let rt = Builder::new_multi_thread().build()?;
    let logic_task = rt.spawn(logic());

    println!("Hello, world!");
    Ok(())
}

#[instrument]
async fn logic() {}
