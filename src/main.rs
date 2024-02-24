use tokio::runtime::Builder;

fn main() -> Result<(), anyhow::Error> {
    let rt = Builder::new_multi_thread().build()?;
    let logic_task = rt.spawn(logic());

    println!("Hello, world!");
    Ok(())
}

async fn logic() {}
