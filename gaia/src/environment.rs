/// possible runtime environment for application
#[derive(PartialEq, Debug)]
pub enum Environment {
    Local,
    Production,
}

#[must_use]
pub fn get_environment() -> Environment {
    std::env::var("APP_ENVIRONMENT")
        .unwrap_or("local".into())
        .try_into()
        .expect("failed to parse APP_ENVIRONMENT")
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str().trim() {
            "local" => {
                dotenvy::dotenv().expect("`.env` not found");
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
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}
