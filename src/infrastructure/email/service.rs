use anyhow::{Context, Result};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use serde::Serialize;

#[derive(Clone)]
enum MailProvider {
    Disabled,
    Resend { api_key: String },
    Smtp {
        mailer: AsyncSmtpTransport<Tokio1Executor>,
    },
}

#[derive(Clone)]
pub struct MailService {
    from_email: String,
    from_name: String,
    provider: MailProvider,
}

#[derive(Debug, Clone)]
pub struct OutboundEmail {
    pub to: String,
    pub subject: String,
    pub html: String,
    pub text: String,
}

#[derive(Serialize)]
struct ResendPayload<'a> {
    from: String,
    to: Vec<&'a str>,
    subject: &'a str,
    html: &'a str,
    text: &'a str,
}

impl MailService {
    pub fn disabled(from_email: String, from_name: String) -> Self {
        Self {
            from_email,
            from_name,
            provider: MailProvider::Disabled,
        }
    }

    pub fn resend(from_email: String, from_name: String, api_key: String) -> Self {
        Self {
            from_email,
            from_name,
            provider: MailProvider::Resend { api_key },
        }
    }

    pub fn smtp(
        from_email: String,
        from_name: String,
        host: String,
        port: u16,
        username: String,
        password: String,
        use_starttls: bool,
    ) -> Result<Self> {
        let creds = Credentials::new(username, password);
        let builder = if use_starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
                .context("invalid smtp host for starttls relay")?
                .port(port)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
                .context("invalid smtp host for relay")?
                .port(port)
        };

        Ok(Self {
            from_email,
            from_name,
            provider: MailProvider::Smtp {
                mailer: builder.credentials(creds).build(),
            },
        })
    }

    pub async fn send(&self, email: OutboundEmail) -> Result<()> {
        match &self.provider {
            MailProvider::Disabled => Ok(()),
            MailProvider::Resend { api_key } => {
                let payload = ResendPayload {
                    from: format!("{} <{}>", self.from_name, self.from_email),
                    to: vec![email.to.as_str()],
                    subject: &email.subject,
                    html: &email.html,
                    text: &email.text,
                };
                let client = reqwest::Client::new();
                let response = client
                    .post("https://api.resend.com/emails")
                    .bearer_auth(api_key)
                    .json(&payload)
                    .send()
                    .await
                    .context("failed to send resend request")?;
                if !response.status().is_success() {
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("resend email failed: {body}");
                }
                Ok(())
            }
            MailProvider::Smtp { mailer } => {
                let message = Message::builder()
                    .from(Mailbox::new(
                        Some(self.from_name.clone()),
                        self.from_email.parse().context("invalid from email")?,
                    ))
                    .to(email.to.parse().context("invalid recipient email")?)
                    .subject(email.subject)
                    .header(ContentType::TEXT_HTML)
                    .body(email.html)
                    .context("failed to build smtp message")?;
                mailer.send(message).await.context("failed to send smtp message")?;
                Ok(())
            }
        }
    }

    pub fn verification_email(&self, to: String, full_name: &str, verification_link: &str) -> OutboundEmail {
        OutboundEmail {
            to,
            subject: "Verify your VeriNest email".to_string(),
            text: format!(
                "Hello {full_name}, verify your email by opening this link: {verification_link}"
            ),
            html: format!(
                "<p>Hello {full_name},</p><p>Verify your email by clicking <a href=\"{verification_link}\">this link</a>.</p>"
            ),
        }
    }

    pub fn kyc_status_email(
        &self,
        to: String,
        full_name: &str,
        verification_status: &str,
        notes: Option<&str>,
    ) -> OutboundEmail {
        let notes = notes.unwrap_or("No additional notes were provided.");
        let subject = match verification_status {
            "verified" => "Your VeriNest KYC has been approved",
            "rejected" => "Your VeriNest KYC needs attention",
            _ => "Your VeriNest KYC status was updated",
        };
        OutboundEmail {
            to,
            subject: subject.to_string(),
            text: format!(
                "Hello {full_name}, your KYC status is now {verification_status}. Notes: {notes}"
            ),
            html: format!(
                "<p>Hello {full_name},</p><p>Your KYC status is now <strong>{verification_status}</strong>.</p><p>{notes}</p>"
            ),
        }
    }
}
