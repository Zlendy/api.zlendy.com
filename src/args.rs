use clap::Parser;

#[derive(Parser, Debug, Clone, Default)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(env = "HOST", long, default_value_t = String::from("0.0.0.0"))]
    pub host: String,

    #[arg(env = "PORT", long, default_value_t = 3000)]
    pub port: u16,

    #[arg(env = "ACCESS_CONTROL_ALLOW_ORIGIN", long)]
    pub access_control_allow_origin: Option<String>,

    #[arg(env = "UMAMI_URL", long)]
    pub umami_url: String,

    #[arg(env = "UMAMI_USERNAME", long)]
    pub umami_username: String,

    #[arg(env = "UMAMI_PASSWORD", long)]
    pub umami_password: String,

    #[arg(env = "UMAMI_WEBSITE_ID", long)]
    pub umami_website_id: String,

    #[arg(env = "FEDIVERSE_URL", long)]
    pub fediverse_url: String,

    #[arg(env = "FEDIVERSE_USER_ID", long)]
    pub fediverse_user_id: String,

    #[arg(env = "ZLENDY_URL", long)]
    pub zlendy_url: String,
}

impl Args {
    pub fn load() -> Result<Args, dotenvy::Error> {
        match dotenvy::dotenv() {
            Ok(_) => log::info!("Found .env file"),
            Err(_) => log::error!(
                "Failed to load .env file, using environment variables and arguments instead"
            ),
        }
        Ok(Args::parse())
    }
}
