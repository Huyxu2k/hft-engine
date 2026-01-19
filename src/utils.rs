

// API Configuration
#[derive(Debug, Clone)]
pub struct ExchangeConfig {
    pub api_key: String,
    pub secret_key: String,
    pub base_url: String,
    pub testnet: bool
}