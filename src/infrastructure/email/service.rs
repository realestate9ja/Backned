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
            subject: "Verify your Verinest email".to_string(),
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
        <div style="font-size:9px;letter-spacing:0.2em;text-transform:uppercase;color:#C4714A;background:#F0E0D4;padding:5px 10px;border-radius:20px;">Security</div>
      </div>
      <div style="padding:40px 40px 36px;background:#C4714A;">
        <p style="font-size:9px;letter-spacing:0.28em;text-transform:uppercase;margin:0 0 12px;color:rgba(255,255,255,0.68);">Email Verification</p>
        <h1 style="font-family:'Cormorant Garamond',serif;font-size:34px;font-weight:600;line-height:1.15;margin:0 0 12px;color:#FFFFFF;">Welcome to<br>Veri<em>nest.</em></h1>
        <p style="font-size:13px;line-height:1.65;margin:0;color:rgba(255,255,255,0.78);">Before you continue, verify your email address so we can secure your account and protect your activity on Verinest.</p>
      </div>
      <div style="padding:36px 40px;">
        <p style="font-size:15px;font-weight:500;color:#1A1814;margin:0 0 14px;">Hi {full_name},</p>
        <p style="font-size:13.5px;color:#5A5248;line-height:1.75;margin:0 0 16px;">Your account has been created successfully. To activate it fully, confirm your email address using the secure link below.</p>
        <div style="margin:28px 0;">
          <a href="{verification_link}" style="display:inline-block;background:#C4714A;color:#ffffff;text-decoration:none;font-size:13px;font-weight:500;letter-spacing:0.05em;padding:15px 32px;border-radius:12px;">Verify My Email</a>
        </div>
        <div style="height:1px;background:#EFE9E2;margin:28px 0;"></div>
        <p style="font-size:12px;color:#9A8F84;line-height:1.7;margin:0;">If the button doesn't work, copy and paste this link in your browser:<br><span style="color:#1A1814;word-break:break-all;font-size:11px;">{verification_link}</span></p>
      </div>
      <div style="padding:24px 40px 30px;background:#FAF7F3;border-top:1px solid #EDE8E0;">
        <div style="font-family:'Cormorant Garamond',serif;font-size:18px;font-weight:600;color:#1A1814;margin-bottom:10px;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
        <p style="font-size:11px;line-height:1.7;color:#9A8F84;margin:0;">If you have any questions, reply to this email directly. A real person will respond — not a bot.</p>
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
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style type="text/css">
* {{ margin: 0; padding: 0; }}
body {{ margin: 0; padding: 20px 0; background-color: #E8E2DA; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; }}
table {{ border-collapse: collapse; width: 100%; }}
.container {{ max-width: 580px; margin: 0 auto; background: #FFFFFF; border-radius: 16px; overflow: hidden; box-shadow: 0 4px 20px rgba(0,0,0,0.08); }}
.header {{ background: #FAF7F3; padding: 24px 32px; border-bottom: 1px solid #EDE8E0; }}
.header-inner {{ display: flex; justify-content: space-between; align-items: center; }}
.logo-section {{ display: flex; align-items: center; gap: 12px; }}
.logo {{ width: 44px; height: 44px; background: #C4714A; border-radius: 8px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }}
.brand {{ font-family: Georgia, 'Times New Roman', serif; font-size: 24px; font-weight: bold; color: #1A1814; }}
.brand em {{ font-style: italic; color: #C4714A; }}
.badge {{ font-size: 10px; letter-spacing: 0.15em; text-transform: uppercase; color: #C4714A; background: #FEF1E8; padding: 6px 12px; border-radius: 20px; font-weight: bold; white-space: nowrap; }}
.hero {{ background: #C4714A; padding: 40px 32px; color: white; text-align: left; }}
.hero-tag {{ font-size: 10px; letter-spacing: 0.15em; text-transform: uppercase; margin: 0 0 16px; color: rgba(255,255,255,0.8); font-weight: bold; display: block; }}
.hero h1 {{ font-family: Georgia, 'Times New Roman', serif; font-size: 40px; font-weight: bold; margin: 0 0 16px; line-height: 1.2; }}
.hero h1 em {{ font-style: italic; }}
.hero p {{ font-size: 14px; line-height: 1.6; margin: 0; color: rgba(255,255,255,0.85); }}
.content {{ padding: 36px 32px; font-size: 13px; line-height: 1.75; color: #5A5248; }}
.content p {{ margin: 0 0 16px; }}
.content p.greeting {{ font-size: 14px; font-weight: bold; color: #1A1814; margin-bottom: 12px; }}
.cta-button {{ display: inline-block; background: #C4714A; color: #FFFFFF; text-decoration: none; font-size: 13px; font-weight: bold; padding: 14px 28px; border-radius: 8px; margin: 24px 0; }}
.cta-button:hover {{ background: #B85C38; }}
.divider {{ height: 1px; background: #EDE8E0; margin: 24px 0; }}
.footer-section {{ padding: 24px 32px; background: #161412; border-top: 1px solid #2A2520; }}
.footer-brand {{ font-family: Georgia, 'Times New Roman', serif; font-size: 20px; font-weight: bold; color: #FFFFFF; margin-bottom: 8px; }}
.footer-brand em {{ font-style: italic; color: #C4714A; }}
.footer-text {{ font-size: 12px; line-height: 1.7; color: rgba(255,255,255,0.7); margin: 0 0 12px; }}
.copyright {{ font-size: 11px; color: rgba(255,255,255,0.5); margin: 12px 0 0; padding-top: 12px; border-top: 1px solid rgba(255,255,255,0.1); }}
</style>
</head>
<body>
<div class="container">
<div class="header">
<div class="logo-section" style="display:flex;align-items:center;gap:12px;">
<div class="logo" style="width:44px;height:44px;background:#C4714A;border-radius:8px;display:flex;align-items:center;justify-content:center;flex-shrink:0;">
<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M3 11L12 3L21 11" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
<rect x="9" y="13" width="6" height="8" rx="1" fill="white"/>
</svg>
</div>
<div class="brand">Veri<em>nest</em></div>
</div>
<div class="badge">Welcome</div>
</div>
<div class="hero">
<div style="font-size:10px;letter-spacing:0.15em;text-transform:uppercase;margin:0 0 16px;color:rgba(255,255,255,0.8);font-weight:bold;">You're in</div>
<h1 style="font-family:Georgia,'Times New Roman',serif;font-size:40px;font-weight:bold;margin:0 0 16px;line-height:1.2;">Welcome to<br>Veri<em>nest.</em></h1>
<p style="font-size:14px;line-height:1.6;margin:0;color:rgba(255,255,255,0.85);">Nigeria's verified rental marketplace, where serious seekers meet verified providers.</p>
</div>
<div class="content">
<p class="greeting">Hi {full_name},</p>
<p>You've just joined a platform built to make renting in Nigeria less risky, less stressful, and much faster.</p>
<p>Post what you need, receive responses from verified providers, and manage everything from one place.</p>
<div style="margin:24px 0;"><a href="{action_url}" style="display:inline-block;background:#C4714A;color:#FFFFFF;text-decoration:none;font-size:13px;font-weight:bold;padding:14px 28px;border-radius:8px;">Open My Dashboard →</a></div>
<div style="height:1px;background:#EDE8E0;margin:24px 0;"></div>
<p>If you have any questions getting started, reply to this email directly. A real person will respond.</p>
</div>
<div class="footer-section">
<div class="footer-brand">Veri<em>nest</em></div>
<p class="footer-text">Verinest helps seekers, agents, and landlords connect through verified listings, clearer pricing, and faster rental matching across Nigeria.</p>
<p class="copyright">© 2025 Verinest. All rights reserved. Nigeria</p>
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
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style type="text/css">
* {{ margin: 0; padding: 0; }}
body {{ margin: 0; padding: 20px 0; background-color: #E8E2DA; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; }}
.container {{ max-width: 580px; margin: 0 auto; background: #FFFFFF; border-radius: 16px; overflow: hidden; }}
.header {{ background: #FAF7F3; padding: 24px 32px; border-bottom: 1px solid #EDE8E0; }}
.logo-section {{ display: flex; align-items: center; gap: 12px; }}
.logo {{ width: 44px; height: 44px; background: #C4714A; border-radius: 8px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }}
.brand {{ font-family: Georgia, 'Times New Roman', serif; font-size: 24px; font-weight: bold; color: #1A1814; }}
.brand em {{ font-style: italic; color: #C4714A; }}
.badge {{ font-size: 10px; letter-spacing: 0.15em; text-transform: uppercase; color: #C4714A; background: #FEF1E8; padding: 6px 12px; border-radius: 20px; font-weight: bold; white-space: nowrap; }}
.hero {{ background: #161412; padding: 40px 32px; color: white; }}
.hero-tag {{ font-size: 10px; letter-spacing: 0.15em; text-transform: uppercase; margin: 0 0 16px; color: rgba(255,255,255,0.8); font-weight: bold; display: block; }}
.hero h1 {{ font-family: Georgia, 'Times New Roman', serif; font-size: 40px; font-weight: bold; margin: 0 0 16px; line-height: 1.2; }}
.hero h1 em {{ font-style: italic; }}
.hero p {{ font-size: 14px; line-height: 1.6; margin: 0; color: rgba(255,255,255,0.85); }}
.content {{ padding: 36px 32px; font-size: 13px; line-height: 1.75; color: #5A5248; }}
.content p {{ margin: 0 0 16px; }}
.content p.greeting {{ font-size: 14px; font-weight: bold; color: #1A1814; margin-bottom: 12px; }}
.code-box {{ background: #FAF7F3; border: 2px solid #EDE8E0; border-radius: 12px; padding: 32px 24px; text-align: center; margin: 28px 0; }}
.code-label {{ font-size: 10px; letter-spacing: 0.15em; text-transform: uppercase; color: #9A8F84; margin: 0 0 12px; font-weight: bold; display: block; }}
.code-display {{ font-family: 'Courier New', monospace; font-size: 48px; font-weight: bold; color: #C4714A; letter-spacing: 0.12em; margin: 0; }}
.code-exp {{ font-size: 12px; color: #9A8F84; margin: 12px 0 0; }}
.warning {{ background: #FFF8F0; border-left: 4px solid #C4714A; padding: 16px 18px; margin: 20px 0; border-radius: 0 8px 8px 0; }}
.warning p {{ font-size: 13px; color: #7A4020; line-height: 1.6; margin: 0; }}
.divider {{ height: 1px; background: #EDE8E0; margin: 24px 0; }}
.footer-section {{ padding: 24px 32px; background: #161412; border-top: 1px solid #2A2520; }}
.footer-brand {{ font-family: Georgia, 'Times New Roman', serif; font-size: 20px; font-weight: bold; color: #FFFFFF; margin-bottom: 8px; }}
.footer-brand em {{ font-style: italic; color: #C4714A; }}
.footer-text {{ font-size: 12px; line-height: 1.7; color: rgba(255,255,255,0.7); margin: 0; }}
.copyright {{ font-size: 11px; color: rgba(255,255,255,0.5); margin: 12px 0 0; padding-top: 12px; border-top: 1px solid rgba(255,255,255,0.1); }}
</style>
</head>
<body>
<div class="container">
<div class="header" style="display:flex;justify-content:space-between;align-items:center;">
<div class="logo-section" style="display:flex;align-items:center;gap:12px;">
<div class="logo" style="width:44px;height:44px;background:#C4714A;border-radius:8px;display:flex;align-items:center;justify-content:center;flex-shrink:0;">
<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M3 11L12 3L21 11" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
<rect x="9" y="13" width="6" height="8" rx="1" fill="white"/>
</svg>
</div>
<div class="brand">Veri<em>nest</em></div>
</div>
<div class="badge">Security</div>
</div>
<div class="hero">
<div style="font-size:10px;letter-spacing:0.15em;text-transform:uppercase;margin:0 0 16px;color:rgba(255,255,255,0.8);font-weight:bold;">Verification Code</div>
<h1 style="font-family:Georgia,'Times New Roman',serif;font-size:40px;font-weight:bold;margin:0 0 16px;line-height:1.2;">Confirm it's<br><em>really you.</em></h1>
<p style="font-size:14px;line-height:1.6;margin:0;color:rgba(255,255,255,0.85);">Use the code below to verify your email address and secure your account.</p>
</div>
<div class="content">
<p class="greeting">Hi {full_name},</p>
<p>You requested a verification code for your Verinest account. Enter this code in the app to continue.</p>
<div class="code-box">
<span class="code-label">Your verification code</span>
<div class="code-display">{code}</div>
<p class="code-exp">Expires in 10 minutes · Do not share this code</p>
</div>
<div class="warning">
<p>⚠ Verinest will never ask for this code by phone or WhatsApp. If someone is asking for it, this is a scam attempt. Do not share it.</p>
</div>
<p>If you didn't request this code, you can safely ignore this email. Your account remains secure.</p>
</div>
<div class="footer-section">
<div class="footer-brand">Veri<em>nest</em></div>
<p class="footer-text">This is an automated security email from Verinest. If you have concerns about your account security, contact security@verinest.ng</p>
<p class="copyright">© 2025 Verinest. All rights reserved. Nigeria</p>
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

     pub fn agent_match_email(
        &self,
        to: String,
        full_name: &str,
        property_title: &str,
        property_price: &str,
        agent_name: &str,
        agent_title: &str,
        action_url: &str,
    ) -> OutboundEmail {
        OutboundEmail {
            to,
            subject: "A verified agent responded to your need".to_string(),
            text: format!(
                "Hi {full_name}, {agent_name} responded to your need for {property_title}. View the listing: {action_url}"
            ),
            html: format!(
                "<!DOCTYPE html>\n<html lang=\"en\" style=\"margin:0;padding:0;\">\n  <body style=\"margin:0;padding:20px;background:#E8E2DA;font-family:'DM Sans',Arial,sans-serif;\">\n    <div style=\"max-width:560px;margin:0 auto;background:#ffffff;border-radius:16px;overflow:hidden;box-shadow:0 4px 20px rgba(0,0,0,0.08);\">\n      <div style=\"background:#F9F6F1;padding:24px 32px;border-bottom:1px solid #EEE8E1;display:flex;align-items:center;justify-content:space-between;\">\n        <div style=\"display:flex;align-items:center;gap:12px;\">\n          <div style=\"width:40px;height:40px;background:#C4714A;border-radius:8px;display:flex;align-items:center;justify-content:center;flex-shrink:0;\">\n            <svg width=\"22\" height=\"22\" viewBox=\"0 0 24 24\" fill=\"none\">\n              <path d=\"M3 11L12 3l9 8\" stroke=\"white\" stroke-width=\"2.2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>\n              <rect x=\"9\" y=\"13\" width=\"6\" height=\"8\" rx=\"3\" fill=\"white\"/>\n            </svg>\n          </div>\n          <div style=\"font-family:'Cormorant Garamond',serif;font-size:20px;font-weight:600;color:#1A1814;letter-spacing:-0.3px;\">Veri<span style=\"font-style:italic;color:#C4714A;\">nest</span></div>\n        </div>\n        <div style=\"font-size:8px;letter-spacing:0.15em;text-transform:uppercase;color:#C4714A;background:#FEF1E8;padding:6px 12px;border-radius:20px;font-weight:500;\">New Match</div>\n      </div>\n      <div style=\"padding:20px 32px;border-bottom:1px solid #EEE8E1;background:#FAFAF8;\">\n        <p style=\"font-size:8px;letter-spacing:0.2em;text-transform:uppercase;margin:0 0 8px;color:#9A8F84;font-weight:500;\">You have a match</p>\n        <h2 style=\"font-family:'Cormorant Garamond',serif;font-size:28px;font-weight:600;line-height:1.2;margin:0;color:#1A1814;\">A verified agent<br>responded to <em>your need.</em></h2>\n      </div>\n      <div style=\"padding:32px;\">\n        <p style=\"font-size:14px;font-weight:500;color:#1A1814;margin:0 0 4px;\">Hi {full_name},</p>\n        <p style=\"font-size:13px;color:#5A5248;line-height:1.75;margin:0 0 20px;\">Good news — a verified agent responded to your need post for a <strong>{property_title}</strong>. Here's what they sent:</p>\n        <div style=\"background:#FAF7F3;border-radius:12px;padding:18px;margin:20px 0;border:1px solid #EEE8E1;\">\n          <div style=\"display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:14px;\">\n            <h3 style=\"font-family:'Cormorant Garamond',serif;font-size:20px;font-weight:600;margin:0;color:#1A1814;flex:1;\">Property Name</h3>\n            <span style=\"background:#C4714A;color:white;font-size:8px;padding:4px 8px;border-radius:4px;text-transform:uppercase;font-weight:600;\">Verified Listing</span>\n          </div>\n          <p style=\"font-size:13px;color:#9A8F84;margin:0 0 10px;\">Location Details</p>\n          <p style=\"font-size:14px;font-weight:600;color:#1A1814;margin:0 0 12px;\">{property_price}</p>\n          <p style=\"font-size:12px;color:#5A5248;margin:0;\">3 bed • 2 bath</p>\n        </div>\n        <div style=\"background:#FAF7F3;border-radius:12px;padding:16px;margin:20px 0;display:flex;gap:12px;border:1px solid #EEE8E1;\">\n          <div style=\"width:40px;height:40px;background:#C4714A;border-radius:50%;flex-shrink:0;display:flex;align-items:center;justify-content:center;\">\n            <span style=\"color:white;font-weight:600;font-size:12px;\">AG</span>\n          </div>\n          <div style=\"flex:1;\">\n            <p style=\"font-size:13px;font-weight:600;color:#1A1814;margin:0 0 2px;\">{agent_name}</p>\n            <p style=\"font-size:12px;color:#9A8F84;margin:0;\">{agent_title}</p>\n            <p style=\"font-size:11px;color:#C4714A;margin:4px 0 0;font-weight:500;\">● VERIFIED PROVIDER</p>\n          </div>\n        </div>\n        <div style=\"margin:24px 0;text-align:center;\">\n          <a href=\"{action_url}\" style=\"display:inline-block;background:#C4714A;color:#ffffff;text-decoration:none;font-size:13px;font-weight:600;padding:12px 28px;border-radius:8px;letter-spacing:0.04em;margin-right:10px;\">View Full Listing →</a>\n          <a href=\"{action_url}\" style=\"display:inline-block;background:transparent;color:#C4714A;text-decoration:none;font-size:13px;font-weight:600;padding:12px 28px;border:1.5px solid #C4714A;border-radius:8px;letter-spacing:0.04em;\">Schedule Viewing</a>\n        </div>\n        <div style=\"height:1px;background:#EEE8E1;margin:20px 0;\"></div>\n        <p style=\"font-size:11px;color:#9A8F84;line-height:1.7;margin:0;\">This agent is verified on Verinest. All communication and viewings should happen through the platform — do not send money or sign anything outside of Verinest.</p>\n      </div>\n      <div style=\"padding:24px 32px 30px;background:#1A1A1A;border-top:1px solid #EEE8E1;\">\n        <div style=\"font-family:'Cormorant Garamond',serif;font-size:16px;font-weight:600;color:#ffffff;margin-bottom:8px;\">Veri<span style=\"font-style:italic;color:#C4714A;\">nest</span></div>\n        <p style=\"font-size:11px;line-height:1.7;color:rgba(255,255,255,0.65);margin:0;\">Matches expire after 48 hours if not reviewed. Log in to your Verinest dashboard to see all your active matches.</p>\n        <div style=\"display:flex;gap:16px;margin-top:12px;\">\n          <a href=\"#\" style=\"font-size:11px;color:#C4714A;text-decoration:none;\">My Dashboard</a>\n          <a href=\"#\" style=\"font-size:11px;color:#C4714A;text-decoration:none;\">My Matches</a>\n          <a href=\"#\" style=\"font-size:11px;color:#C4714A;text-decoration:none;\">Help Centre</a>\n          <a href=\"#\" style=\"font-size:11px;color:#C4714A;text-decoration:none;\">Unsubscribe</a>\n        </div>\n        <p style=\"font-size:10px;color:rgba(255,255,255,0.45);margin:12px 0 0;border-top:1px solid rgba(255,255,255,0.1);padding-top:12px;\">© 2025 Verinest. All rights reserved. Nigeria</p>\n      </div>\n    </div>\n  </body>\n</html>"
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

   
