use std::error::Error;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(env = "HOST", long, default_value_t = String::from("0.0.0.0"))]
    pub host: String,

    #[arg(env = "PORT", long, default_value_t = 3000)]
    pub port: u16,

    #[arg(env = "UMAMI_URL", long)]
    pub umami_url: String,

    #[arg(env = "UMAMI_USERNAME", long)]
    pub umami_username: String,

    #[arg(env = "UMAMI_PASSWORD", long)]
    pub umami_password: String,

    #[arg(env = "ZLENDY_URL", long)]
    pub zlendy_url: String,
}

impl Args {
    pub fn load() -> Result<Args, Box<dyn Error>> {
        dotenvy::dotenv()?;
        Ok(Args::parse())
    }
}
