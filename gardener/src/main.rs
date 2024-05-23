use gaia::{environment::get_environment, get_connection_pool, get_settings, init_tracing};
use gardener::start_app;

#[tokio::main]
async fn main() {
    let env = get_environment();
    let settings = get_settings(&env).expect("failed to parse settings");
    let pool = get_connection_pool(&env, &settings.database).await;

    init_tracing(&env, vec![("gardener")]);
    start_app(settings, pool).await;
}
