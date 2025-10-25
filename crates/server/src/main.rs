use server::configuration::get_configuration;
use server::startup::Application;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let application = Application::build(configuration).await?;
    application.run().await?;
    Ok(())
}


