use dotenvy::dotenv;

#[derive(PartialEq)]
pub enum Environment {
    Local,
    Production,
}

pub fn get_environment() -> Environment {
    std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("failed to parse APP_ENVIRONMENT")
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str().trim() {
            "local" => {
                dotenv().expect(".env file not found");
                Ok(Self::Local)
            }
            "production" => Ok(Self::Production),
            unknown => Err(format!(
                "{unknown} is not a supported environment. use either `local` or `production`"
            )),
        }
    }
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}
