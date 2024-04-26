use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt};

use crate::settings::environment::Environment;

pub fn init_tracing(env: &Environment) {
    let env_layer = match *env {
        Environment::Local => EnvFilter::from_default_env()
            .add_directive(
                format!("telebot={}", LevelFilter::DEBUG)
                    .parse()
                    .expect("unable to set telebot tracing for local runtime"),
            )
            .add_directive(
                format!("tower_http={}", LevelFilter::DEBUG)
                    .parse()
                    .expect("unable to set axum tracing middleware for local runtime"),
            ),
        Environment::Production => EnvFilter::from_default_env().add_directive(
            format!("telebot={}", LevelFilter::INFO)
                .parse()
                .expect("unable to set telebot tracing for production runtime"),
        ),
    };

    let format_layer = fmt::layer()
        .without_time()
        .with_file(true)
        .with_line_number(true)
        .with_target(false);

    let subscriber = tracing_subscriber::registry()
        .with(env_layer)
        .with(format_layer);

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");

    let env_name = env.as_str();
    tracing::info!("telebot app started in {env_name} environment!");
}

#[cfg(test)]
mod test {

    #[derive(Debug, thiserror::Error)]
    enum TestError {
        #[error("effwgrgw")]
        HelloError(String),
    }

    fn hello() -> Result<(), TestError> {
        Err(TestError::HelloError("evfefe2f".to_string()))
    }
    #[test]
    fn error() {
        match hello() {
            Ok(_v) => println!("hello"),
            Err(e) => println!("{e}"),
        }
        let _ = hello().map_err(|e| println!("{e}"));
        hello().expect("csddc");
        // hello().unwrap();
    }
}
