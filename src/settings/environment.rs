pub enum Environment {
    Local,
    Production,
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
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
            Environment::Production => "local",
        }
    }
}
