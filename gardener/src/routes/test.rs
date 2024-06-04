//! this module is for api experimentation
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use tower_sessions::{
    cookie::{Cookie, Expiration},
    Session,
};

use async_trait::async_trait;

use super::AppState;

#[utoipa::path(get, path = "/test/cookie_handler", tag = "test")]
pub async fn cookie_handler(jar: CookieJar) -> Result<(CookieJar, String), StatusCode> {
    let zz = Cookie::build(("turtle", "check me in dev tools")).expires(Expiration::Session);
    Ok((jar.add(zz), "check the damn cookie".to_string()))
}

const COUNTER_KEY: &str = "counter";

#[derive(Serialize, Deserialize, Default)]
struct Counter(usize);

#[utoipa::path(get, path = "/test/session_handler", tag = "test")]
pub async fn session_handler(session: Session) -> impl IntoResponse {
    let counter: Counter = session.get(COUNTER_KEY).await.unwrap().unwrap_or_default();
    session.insert(COUNTER_KEY, counter.0 + 1).await.unwrap();
    format!("Current count: {}", counter.0)
}
#[derive(Clone, Deserialize, Serialize)]
struct GuestData {
    random: usize,
    wtf: usize,
    zz: Option<usize>,
    pageviews: usize,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
}

impl Default for GuestData {
    fn default() -> Self {
        Self {
            random: 0,
            wtf: 1,
            zz: None,
            pageviews: 0,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
        }
    }
}

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct GuestData2 {
    haha: bool,
    init: usize,
}

#[allow(clippy::struct_field_names)]
pub struct Guest {
    session: Session,
    guest_data: GuestData,
    guest_data_2: GuestData2,
}

impl Guest {
    const GUEST_DATA_KEY: &'static str = "guest.data";

    fn first_seen(&self) -> DateTime<Utc> {
        self.guest_data.first_seen
    }

    fn last_seen(&self) -> DateTime<Utc> {
        self.guest_data.last_seen
    }

    fn pageviews(&self) -> usize {
        self.guest_data.pageviews
    }

    async fn mark_pageview(&mut self) {
        self.guest_data.pageviews += 1;
        Self::update_session(&self.session, &self.guest_data).await;
    }

    async fn update_session(session: &Session, guest_data: &GuestData) {
        session
            .insert(Self::GUEST_DATA_KEY, guest_data.clone())
            .await
            .unwrap();
    }

    const GUEST_DATA_KEY_2: &'static str = "guest.data_2";
    async fn update_session_2(session: &Session, guest_data: &GuestData2) {
        session
            .insert(Self::GUEST_DATA_KEY_2, guest_data.clone())
            .await
            .unwrap();
    }
    async fn increase_init(&mut self) {
        self.guest_data_2.init += 1;
        Self::update_session_2(&self.session, &self.guest_data_2).await;
    }
}

impl fmt::Debug for Guest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Guest")
            .field("pageviews", &self.pageviews())
            .field("first_seen", &self.first_seen())
            .field("last_seen", &self.last_seen())
            .field("random", &self.guest_data.random)
            .field("wtf", &self.guest_data.wtf)
            .field("zz", &self.guest_data.zz)
            .field("init", &self.guest_data_2.init)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Guest
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;

        let mut guest_data: GuestData = session
            .get(Self::GUEST_DATA_KEY)
            .await
            .unwrap_or_default()
            .unwrap_or_default();

        guest_data.last_seen = Utc::now();

        Self::update_session(&session, &guest_data).await;

        let guest_data_2: GuestData2 = session
            .get(Self::GUEST_DATA_KEY_2)
            .await
            .unwrap_or_default()
            .unwrap_or_default();

        Ok(Self {
            session,
            guest_data,
            guest_data_2,
        })
    }
}

/// test for strongly typed session
///
/// if developer wants to include new field, he shouldn't touch `GuestData` and `GuestData2`.
///
/// add 1 more field to `Guest` such as `GuestData3`.
///
#[utoipa::path(get, path = "/test/session_strongly_typed", tag = "test")]
pub async fn session_strongly_typed(mut guest: Guest) -> impl IntoResponse {
    guest.mark_pageview().await;
    guest.increase_init().await;
    format!("{guest:#?}")
}

pub fn test_router() -> Router<AppState> {
    Router::new()
        .route("/cookie_handler", get(cookie_handler))
        .route("/session_handler", get(session_handler))
        .route("/session_strongly_typed", get(session_strongly_typed))
}
