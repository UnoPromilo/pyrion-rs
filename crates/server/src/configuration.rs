use serde::Deserialize;

pub fn get_configuration() -> Result<Configuration, config::ConfigError> {
    let configuration = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    configuration.try_deserialize::<Configuration>()
}

#[derive(Deserialize)]
pub struct Configuration {
    pub application: ApplicationConfiguration,
}

#[derive(Deserialize)]
pub struct ApplicationConfiguration {
    pub port: u16,
    pub host: String,
}
