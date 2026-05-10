pub type Result<T> = std::result::Result<T, TartsError>;

#[derive(Debug, thiserror::Error)]
pub enum TartsError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    // #[error("System error: {0}")]
    // System(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to deserialize config: {0}")]
    DeserializeFormat(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    SerializeFormat(#[from] toml::ser::Error),
}
