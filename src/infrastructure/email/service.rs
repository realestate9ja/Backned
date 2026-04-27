use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Utc};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use serde::Serialize;

#[derive(Clone)]
enum MailProvider {
    Disabled,
    Resend {
        api_key: String,
    },
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
                mailer
                    .send(message)
                    .await
                    .context("failed to send smtp message")?;
                Ok(())
            }
        }
    }

    pub fn verification_email(
        &self,
        to: String,
        full_name: &str,
        verification_link: &str,
    ) -> OutboundEmail {
        OutboundEmail {
            to,
            subject: "Verify your VeriNest email".to_string(),
            text: format!(
                "Hello {full_name}, verify your email by opening this link: {verification_link}"
            ),
            html: format!(
                r#"<!DOCTYPE html>
<html lang="en">
  <body style="margin:0;padding:24px;background:#E8E2DA;font-family:'DM Sans',Arial,sans-serif;">
    <div style="max-width:560px;margin:0 auto;background:#ffffff;border-radius:20px;overflow:hidden;box-shadow:0 8px 40px rgba(0,0,0,0.10);">
      <div style="background:#FAF7F3;padding:28px 40px 24px;border-bottom:1px solid #EDE8E0;display:flex;align-items:center;justify-content:space-between;">
        <div style="display:flex;align-items:center;gap:10px;">
          <div style="width:36px;height:36px;background:#C4714A;border-radius:10px;display:flex;align-items:center;justify-content:center;">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
            </svg>
          </div>
          <div style="font-family:'Cormorant Garamond',serif;font-size:22px;font-weight:600;color:#1A1814;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        </div>
        <div style="font-size:9px;letter-spacing:0.2em;text-transform:uppercase;color:#C4714A;background:#F0E0D4;padding:5px 10px;border-radius:20px;">Welcome</div>
      </div>
      <div style="padding:40px 40px 36px;background:#C4714A;">
        <p style="font-size:9px;letter-spacing:0.28em;text-transform:uppercase;margin:0 0 12px;color:rgba(255,255,255,0.68);">You're in</p>
        <h1 style="font-family:'Cormorant Garamond',serif;font-size:34px;font-weight:600;line-height:1.15;margin:0 0 12px;color:#FFFFFF;">Welcome to<br>Veri<em>nest.</em></h1>
        <p style="font-size:13px;line-height:1.65;margin:0;color:rgba(255,255,255,0.78);">Before you continue, verify your email address so we can secure your account and protect your activity on Verinest.</p>
      </div>
      <div style="padding:36px 40px;">
        <p style="font-size:15px;font-weight:500;color:#1A1814;margin:0 0 14px;">Hi {full_name},</p>
        <p style="font-size:13.5px;color:#5A5248;line-height:1.75;margin:0 0 16px;">Your account has been created successfully. To activate it fully, confirm your email address using the secure button below.</p>
        <div style="margin:24px 0;">
          <a href="{verification_link}" style="display:inline-block;background:#C4714A;color:#ffffff;text-decoration:none;font-size:12px;font-weight:500;padding:14px 22px;border-radius:999px;letter-spacing:0.04em;">Verify My Email →</a>
        </div>
        <div style="height:1px;background:#EFE9E2;margin:24px 0;"></div>
        <p style="font-size:12px;color:#9A8F84;line-height:1.7;margin:0;">If the button does not work, copy and open this link:<br><span style="color:#1A1814;word-break:break-all;">{verification_link}</span></p>
      </div>
      <div style="padding:24px 40px 30px;background:#FAF7F3;border-top:1px solid #EDE8E0;">
        <div style="font-family:'Cormorant Garamond',serif;font-size:18px;font-weight:600;color:#1A1814;margin-bottom:10px;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        <p style="font-size:11px;line-height:1.7;color:#9A8F84;margin:0;">If you have any questions getting started, reply to this email directly. A real person will respond.</p>
      </div>
    </div>
  </body>
</html>"#
            ),
        }
    }

    pub fn welcome_email(&self, to: String, full_name: &str, action_url: &str) -> OutboundEmail {
        OutboundEmail {
            to,
            subject: "Welcome to VeriNest".to_string(),
            text: format!(
                "Hi {full_name}, welcome to Verinest. Open your dashboard to get started: {action_url}"
            ),
            html: format!(
                r#"<!DOCTYPE html>
<html lang="en">
  <body style="margin:0;padding:24px;background:#E8E2DA;font-family:'DM Sans',Arial,sans-serif;">
    <div style="max-width:560px;margin:0 auto;background:#ffffff;border-radius:20px;overflow:hidden;box-shadow:0 8px 40px rgba(0,0,0,0.10);">
      <div style="background:#FAF7F3;padding:28px 40px 24px;border-bottom:1px solid #EDE8E0;display:flex;align-items:center;justify-content:space-between;">
        <div style="display:flex;align-items:center;gap:10px;">
          <div style="width:36px;height:36px;background:#C4714A;border-radius:10px;display:flex;align-items:center;justify-content:center;">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
            </svg>
          </div>
          <div style="font-family:'Cormorant Garamond',serif;font-size:22px;font-weight:600;color:#1A1814;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        </div>
        <div style="font-size:9px;letter-spacing:0.2em;text-transform:uppercase;color:#C4714A;background:#F0E0D4;padding:5px 10px;border-radius:20px;">Welcome</div>
      </div>
      <div style="padding:40px 40px 36px;background:#C4714A;">
        <p style="font-size:9px;letter-spacing:0.28em;text-transform:uppercase;margin:0 0 12px;color:rgba(255,255,255,0.68);">You're in</p>
        <h1 style="font-family:'Cormorant Garamond',serif;font-size:34px;font-weight:600;line-height:1.15;margin:0 0 12px;color:#FFFFFF;">Welcome to<br>Veri<em>nest.</em></h1>
        <p style="font-size:13px;line-height:1.65;margin:0;color:rgba(255,255,255,0.78);">Nigeria's verified rental marketplace, where serious seekers meet verified providers.</p>
      </div>
      <div style="padding:36px 40px;">
        <p style="font-size:15px;font-weight:500;color:#1A1814;margin:0 0 14px;">Hi {full_name},</p>
        <p style="font-size:13.5px;color:#5A5248;line-height:1.75;margin:0 0 14px;">You've just joined a platform built to make renting in Nigeria less risky, less stressful, and much faster.</p>
        <p style="font-size:13.5px;color:#5A5248;line-height:1.75;margin:0 0 20px;">Post what you need, receive responses from verified providers, and manage everything from one place.</p>
        <div style="margin:24px 0;">
          <a href="{action_url}" style="display:inline-block;background:#C4714A;color:#ffffff;text-decoration:none;font-size:12px;font-weight:500;padding:14px 22px;border-radius:999px;letter-spacing:0.04em;">Open My Dashboard →</a>
        </div>
        <div style="height:1px;background:#EFE9E2;margin:24px 0;"></div>
        <p style="font-size:12px;color:#9A8F84;line-height:1.7;margin:0;">If you have any questions getting started, reply to this email directly. A real person will respond.</p>
      </div>
      <div style="padding:24px 40px 30px;background:#FAF7F3;border-top:1px solid #EDE8E0;">
        <div style="font-family:'Cormorant Garamond',serif;font-size:18px;font-weight:600;color:#1A1814;margin-bottom:10px;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        <p style="font-size:11px;line-height:1.7;color:#9A8F84;margin:0;">Verinest helps seekers, agents, and landlords connect through verified listings, clearer pricing, and faster rental matching across Nigeria.</p>
      </div>
    </div>
  </body>
</html>"#
            ),
        }
    }

    pub fn verification_code_email(
        &self,
        to: String,
        full_name: &str,
        code: &str,
    ) -> OutboundEmail {
        OutboundEmail {
            to,
            subject: "Your VeriNest verification code".to_string(),
            text: format!("Hello {full_name}, your VeriNest verification code is {code}."),
            html: format!(
                r#"<!DOCTYPE html>
<html lang="en">
  <body style="margin:0;padding:24px;background:#E8E2DA;font-family:'DM Sans',Arial,sans-serif;">
    <div style="max-width:560px;margin:0 auto;background:#ffffff;border-radius:20px;overflow:hidden;box-shadow:0 8px 40px rgba(0,0,0,0.10);">
      <div style="background:#FAF7F3;padding:28px 40px 24px;border-bottom:1px solid #EDE8E0;display:flex;align-items:center;justify-content:space-between;">
        <div style="display:flex;align-items:center;gap:10px;">
          <div style="width:36px;height:36px;background:#C4714A;border-radius:10px;display:flex;align-items:center;justify-content:center;">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
            </svg>
          </div>
          <div style="font-family:'Cormorant Garamond',serif;font-size:22px;font-weight:600;color:#1A1814;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        </div>
        <div style="font-size:9px;letter-spacing:0.2em;text-transform:uppercase;color:#C4714A;background:#F0E0D4;padding:5px 10px;border-radius:20px;">Security</div>
      </div>
      <div style="padding:40px 40px 36px;background:#161412;">
        <p style="font-size:9px;letter-spacing:0.28em;text-transform:uppercase;margin:0 0 12px;color:rgba(255,255,255,0.68);">Verification Code</p>
        <h1 style="font-family:'Cormorant Garamond',serif;font-size:34px;font-weight:600;line-height:1.15;margin:0 0 12px;color:#FFFFFF;">Confirm it's<br><em>really you.</em></h1>
        <p style="font-size:13px;line-height:1.65;margin:0;color:rgba(255,255,255,0.78);">Use the code below to verify your email address and secure your account.</p>
      </div>
      <div style="padding:36px 40px;">
        <p style="font-size:15px;font-weight:500;color:#1A1814;margin:0 0 14px;">Hi {full_name},</p>
        <p style="font-size:13.5px;color:#5A5248;line-height:1.75;margin:0 0 18px;">You requested a verification code for your Verinest account. Enter this code in the app to continue.</p>
        <div style="background:#FAF7F3;border:1px solid #EDE8E0;border-radius:18px;padding:24px;text-align:center;margin:0 0 18px;">
          <p style="font-size:10px;letter-spacing:0.2em;text-transform:uppercase;color:#9A8F84;margin:0 0 10px;">Your verification code</p>
          <div style="font-family:'Cormorant Garamond',serif;font-size:38px;font-weight:600;color:#1A1814;letter-spacing:0.18em;">{code}</div>
          <p style="font-size:12px;color:#9A8F84;margin:10px 0 0;">Expires in 10 minutes · Do not share this code</p>
        </div>
        <div style="background:#FFF8F0;border-left:4px solid #C4714A;padding:14px 18px;border-radius:0 10px 10px 0;margin-bottom:20px;">
          <p style="font-size:12px;color:#7A4020;line-height:1.6;margin:0;">Verinest will never ask for this code by phone or WhatsApp. If someone is asking for it, do not share it.</p>
        </div>
        <p style="font-size:12px;color:#9A8F84;line-height:1.7;margin:0;">If you didn't request this code, you can safely ignore this email. Your account remains secure.</p>
      </div>
      <div style="padding:24px 40px 30px;background:#FAF7F3;border-top:1px solid #EDE8E0;">
        <div style="font-family:'Cormorant Garamond',serif;font-size:18px;font-weight:600;color:#1A1814;margin-bottom:10px;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        <p style="font-size:11px;line-height:1.7;color:#9A8F84;margin:0;">This is an automated security email from Verinest. If you have concerns about your account security, contact security@verinest.ng.</p>
      </div>
    </div>
  </body>
</html>"#
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
        let normalized_status = verification_status.to_lowercase();
        if normalized_status == "verified" || normalized_status == "approved" {
            return OutboundEmail {
                to,
                subject: "Your VeriNest KYC has been approved".to_string(),
                text: format!(
                    "Hello {full_name}, your Verinest verification has been approved. Notes: {notes}"
                ),
                html: format!(
                    r#"<!DOCTYPE html>
<html lang="en">
  <body style="margin:0;padding:24px;background:#E8E2DA;font-family:'DM Sans',Arial,sans-serif;">
    <div style="max-width:560px;margin:0 auto;background:#ffffff;border-radius:20px;overflow:hidden;box-shadow:0 8px 40px rgba(0,0,0,0.10);">
      <div style="background:#FAF7F3;padding:28px 40px 24px;border-bottom:1px solid #EDE8E0;display:flex;align-items:center;justify-content:space-between;">
        <div style="display:flex;align-items:center;gap:10px;">
          <div style="width:36px;height:36px;background:#C4714A;border-radius:10px;display:flex;align-items:center;justify-content:center;">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
            </svg>
          </div>
          <div style="font-family:'Cormorant Garamond',serif;font-size:22px;font-weight:600;color:#1A1814;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        </div>
        <div style="font-size:9px;letter-spacing:0.2em;text-transform:uppercase;color:#3A8A52;background:#EBF7EF;padding:5px 10px;border-radius:20px;">Approved ✓</div>
      </div>
      <div style="padding:40px 40px 36px;background:#C4714A;">
        <p style="font-size:9px;letter-spacing:0.28em;text-transform:uppercase;margin:0 0 12px;color:rgba(255,255,255,0.68);">Verification Complete</p>
        <h1 style="font-family:'Cormorant Garamond',serif;font-size:34px;font-weight:600;line-height:1.15;margin:0 0 12px;color:#FFFFFF;">You're now<br><em>verified.</em></h1>
        <p style="font-size:13px;line-height:1.65;margin:0;color:rgba(255,255,255,0.78);">Your identity and credentials have been confirmed. Your Verinest profile is now trusted across the platform.</p>
      </div>
      <div style="padding:36px 40px;">
        <p style="font-size:15px;font-weight:500;color:#1A1814;margin:0 0 14px;">Hi {full_name},</p>
        <p style="font-size:13.5px;color:#5A5248;line-height:1.75;margin:0 0 14px;">Congratulations — your Verinest verification has been reviewed and approved.</p>
        <div style="background:#FAF7F3;border-left:4px solid #3A8A52;padding:16px 18px;border-radius:0 10px 10px 0;margin:18px 0;">
          <div style="font-size:13px;font-weight:600;color:#3A8A52;margin-bottom:6px;">Your verified badge is active</div>
          <div style="font-size:13px;color:#5A5248;line-height:1.7;">Your profile now carries the trust signal other users see when deciding who to work with on Verinest.</div>
        </div>
        <p style="font-size:12px;color:#9A8F84;line-height:1.7;margin:0;">{notes}</p>
      </div>
      <div style="padding:24px 40px 30px;background:#FAF7F3;border-top:1px solid #EDE8E0;">
        <div style="font-family:'Cormorant Garamond',serif;font-size:18px;font-weight:600;color:#1A1814;margin-bottom:10px;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        <p style="font-size:11px;line-height:1.7;color:#9A8F84;margin:0;">Questions about your profile or next steps? Reply to this email and our team will help.</p>
      </div>
    </div>
  </body>
</html>"#
                ),
            };
        }
        OutboundEmail {
            to,
            subject: subject.to_string(),
            text: format!(
                "Hello {full_name}, your KYC status is now {verification_status}. Notes: {notes}"
            ),
            html: format!(
                r#"<p>Hello {full_name},</p><p>Your KYC status is now <strong>{verification_status}</strong>.</p><p>{notes}</p>"#
            ),
        }
    }

    pub fn schedule_upcoming_email(
        &self,
        to: String,
        recipient_name: &str,
        counterparty_name: &str,
        property_title: &str,
        property_location: &str,
        booking_type: &str,
        scheduled_for: DateTime<Utc>,
        action_url: &str,
        for_provider: bool,
    ) -> OutboundEmail {
        let role_label = if for_provider {
            "Schedule Reminder"
        } else {
            "Upcoming Visit"
        };
        let title = if for_provider {
            "Your schedule starts in\n<em>under one hour.</em>"
        } else {
            "Your viewing starts in\n<em>under one hour.</em>"
        };
        let body_intro = if for_provider {
            format!(
                "You have a {} with <strong>{}</strong> scheduled soon for <strong>{}</strong>.",
                humanize_booking_type(booking_type),
                counterparty_name,
                property_title
            )
        } else {
            format!(
                "Your {} with <strong>{}</strong> is coming up soon for <strong>{}</strong>.",
                humanize_booking_type(booking_type),
                counterparty_name,
                property_title
            )
        };
        let action_copy = if for_provider {
            "Open Calendar"
        } else {
            "Open My Bookings"
        };
        let subject = if for_provider {
            format!("Upcoming schedule in 1 hour — {}", property_title)
        } else {
            format!(
                "Your Verinest schedule starts in 1 hour — {}",
                property_title
            )
        };
        let schedule_label = format_schedule(scheduled_for);

        OutboundEmail {
            to,
            subject,
            text: format!(
                "Hi {recipient_name}, {body_intro} Time: {schedule_label}. Location: {property_location}. Open: {action_url}"
            ),
            html: render_schedule_email_html(
                role_label,
                title,
                recipient_name,
                &body_intro,
                property_title,
                property_location,
                &humanize_booking_type(booking_type),
                &schedule_label,
                action_copy,
                action_url,
                "Please be ready a few minutes early and keep communication inside Verinest for your safety.",
                false,
            ),
        }
    }

    pub fn schedule_past_due_email(
        &self,
        to: String,
        recipient_name: &str,
        counterparty_name: &str,
        property_title: &str,
        property_location: &str,
        booking_type: &str,
        scheduled_for: DateTime<Utc>,
        action_url: &str,
        for_provider: bool,
    ) -> OutboundEmail {
        let role_label = if for_provider {
            "Past Schedule"
        } else {
            "Missed Schedule"
        };
        let title = if for_provider {
            "This schedule is now\n<em>past due.</em>"
        } else {
            "This visit is now\n<em>past due.</em>"
        };
        let body_intro = if for_provider {
            format!(
                "Your {} with <strong>{}</strong> for <strong>{}</strong> passed more than one hour ago.",
                humanize_booking_type(booking_type),
                counterparty_name,
                property_title
            )
        } else {
            format!(
                "Your {} with <strong>{}</strong> for <strong>{}</strong> passed more than one hour ago.",
                humanize_booking_type(booking_type),
                counterparty_name,
                property_title
            )
        };
        let action_copy = if for_provider {
            "Review Schedule"
        } else {
            "Review Booking"
        };
        let subject = if for_provider {
            format!("Past-due schedule — {}", property_title)
        } else {
            format!("Missed Verinest schedule — {}", property_title)
        };
        let schedule_label = format_schedule(scheduled_for);

        OutboundEmail {
            to,
            subject,
            text: format!(
                "Hi {recipient_name}, {body_intro} Original time: {schedule_label}. Location: {property_location}. Open: {action_url}"
            ),
            html: render_schedule_email_html(
                role_label,
                title,
                recipient_name,
                &body_intro,
                property_title,
                property_location,
                &humanize_booking_type(booking_type),
                &schedule_label,
                action_copy,
                action_url,
                "If the meeting happened, update the status in Verinest. If it did not, reschedule inside the platform so both sides stay aligned.",
                true,
            ),
        }
    }
}

fn humanize_booking_type(booking_type: &str) -> String {
    booking_type.replace('_', " ")
}

fn format_schedule(scheduled_for: DateTime<Utc>) -> String {
    let lagos_offset = FixedOffset::east_opt(3600).expect("valid lagos offset");
    let local_time = scheduled_for.with_timezone(&lagos_offset);
    local_time
        .format("%A, %d %B %Y at %I:%M %p WAT")
        .to_string()
}

fn render_schedule_email_html(
    badge: &str,
    title: &str,
    recipient_name: &str,
    intro: &str,
    property_title: &str,
    property_location: &str,
    booking_type: &str,
    schedule_label: &str,
    action_copy: &str,
    action_url: &str,
    footer_note: &str,
    overdue: bool,
) -> String {
    let hero_class = if overdue { "#161412" } else { "#C4714A" };
    let badge_bg = if overdue { "#FFF1EB" } else { "#F0E0D4" };
    let badge_fg = if overdue { "#B85C38" } else { "#C4714A" };
    let card_bg = if overdue { "#FFF8F4" } else { "#FAF7F3" };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
  <body style="margin:0;padding:24px;background:#E8E2DA;font-family:'DM Sans',Arial,sans-serif;">
    <div style="max-width:560px;margin:0 auto;background:#ffffff;border-radius:20px;overflow:hidden;box-shadow:0 8px 40px rgba(0,0,0,0.10);">
      <div style="background:#FAF7F3;padding:28px 40px 24px;border-bottom:1px solid #EDE8E0;display:flex;align-items:center;justify-content:space-between;">
        <div style="display:flex;align-items:center;gap:10px;">
          <div style="width:36px;height:36px;background:#C4714A;border-radius:10px;display:flex;align-items:center;justify-content:center;">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
            </svg>
          </div>
          <div style="font-family:'Cormorant Garamond',serif;font-size:22px;font-weight:600;color:#1A1814;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        </div>
        <div style="font-size:9px;letter-spacing:0.2em;text-transform:uppercase;color:{badge_fg};background:{badge_bg};padding:5px 10px;border-radius:20px;">{badge}</div>
      </div>

      <div style="padding:40px 40px 36px;background:{hero_class};">
        <p style="font-size:9px;letter-spacing:0.28em;text-transform:uppercase;margin:0 0 12px;color:rgba(255,255,255,0.68);">{badge}</p>
        <h1 style="font-family:'Cormorant Garamond',serif;font-size:34px;font-weight:600;line-height:1.15;margin:0 0 12px;color:#FFFFFF;">{title}</h1>
        <p style="font-size:13px;line-height:1.65;margin:0;color:rgba(255,255,255,0.78);">Stay coordinated with your Verinest schedule and handle updates quickly inside the platform.</p>
      </div>

      <div style="padding:36px 40px;">
        <p style="font-size:15px;font-weight:500;color:#1A1814;margin:0 0 14px;">Hi {recipient_name},</p>
        <p style="font-size:13.5px;color:#5A5248;line-height:1.75;margin:0 0 18px;">{intro}</p>

        <div style="background:{card_bg};border:1px solid #EDE8E0;border-radius:18px;padding:18px 20px;margin:0 0 20px;">
          <div style="font-size:10px;letter-spacing:0.2em;text-transform:uppercase;color:#9A8F84;margin-bottom:10px;">Schedule details</div>
          <div style="font-size:17px;font-weight:600;color:#1A1814;margin-bottom:6px;">{property_title}</div>
          <div style="font-size:12px;color:#9A8F84;line-height:1.7;">
            <div><strong style="color:#1A1814;">Type:</strong> {booking_type}</div>
            <div><strong style="color:#1A1814;">Time:</strong> {schedule_label}</div>
            <div><strong style="color:#1A1814;">Location:</strong> {property_location}</div>
          </div>
        </div>

        <div style="margin:0 0 20px;">
          <a href="{action_url}" style="display:inline-block;background:#C4714A;color:#ffffff;text-decoration:none;font-size:12px;font-weight:500;padding:14px 22px;border-radius:999px;letter-spacing:0.04em;">{action_copy} →</a>
        </div>

        <div style="height:1px;background:#EFE9E2;margin:24px 0;"></div>
        <p style="font-size:12px;color:#9A8F84;line-height:1.7;margin:0;">{footer_note}</p>
      </div>

      <div style="padding:24px 40px 30px;background:#FAF7F3;border-top:1px solid #EDE8E0;">
        <div style="font-family:'Cormorant Garamond',serif;font-size:18px;font-weight:600;color:#1A1814;margin-bottom:10px;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        <p style="font-size:11px;line-height:1.7;color:#9A8F84;margin:0;">This is an automated schedule email from Verinest. Please manage visits, confirmations, and reschedules inside the platform.</p>
      </div>
    </div>
  </body>
</html>"#
    )
}
