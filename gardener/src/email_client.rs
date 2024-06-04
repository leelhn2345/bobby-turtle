use std::time::Duration;

use axum::http::{HeaderMap, HeaderValue};
use gaia::email::EmailSettings;
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client,
};
use secrecy::ExposeSecret;
use serde_json::json;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    api: String,
    base_url: String,
}

impl EmailClient {
    pub fn new(settings: EmailSettings) -> Self {
        let timeout = Duration::from_millis(settings.timeout_milliseconds);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            "api-key",
            HeaderValue::from_str(settings.api_key.expose_secret())
                .expect("can't parse into header string value"),
        );
        let http_client = Client::builder()
            .timeout(timeout)
            .default_headers(headers)
            .build()
            .expect("cant build email http client");

        Self {
            http_client,
            api: settings.api,
            base_url: settings.base_url,
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn send_confirmation_email(
        &self,
        name: String,
        email: String,
        token: String,
    ) -> Result<(), reqwest::Error> {
        let confirmation_link = format!("{}/account_verification/{}", self.base_url, token);

        let html_content = format!(
            "<html>
            <head></head>
            <body>
            <p>Hello {name},
            </p>Please click <a href=\"{confirmation_link}\">here</a> to verify your account.</p>
            <p>If you did not sign up with us, please ignore this email.</p>
            <p>Warm Regards,</p>
            <p>Digital Garden</p>
            </body></html>"
        );
        let zzz = json!({
           "sender":{
              "name":"Digital Garden",
              "email":"noreply@alaladin.com"
           },
           "to":[
              {
                 "email":email,
                 "name":name
              }
           ],
           "subject":"Account Verification",
           // "htmlContent":"zzz"
           "htmlContent":html_content
        });
        self.http_client
            .post(self.api.clone())
            .json(&zzz)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
