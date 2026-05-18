use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Utc};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use serde::Serialize;

pub const HEADER_BASE_URL: &str = "https://res.cloudinary.com/dui0hakkq/image/upload";
pub const HEADER_24HR: &str = "header-24hr.svg";
pub const HEADER_CONFIRMED: &str = "header-confirmed.svg";
pub const HEADER_DIGEST: &str = "header-digest.svg";
pub const HEADER_LEAD_ALERT: &str = "header-lead-alert.svg";
pub const HEADER_LEAD_ALERT_ALT: &str = "header-lead-alert-1.svg";
pub const HEADER_NEW_MATCH: &str = "header-new-match.svg";
pub const HEADER_NO_MATCH: &str = "header-no-match.svg";
pub const HEADER_POST_VIEWING: &str = "header-post-viewing.svg";
pub const HEADER_REQUEST_LIVE: &str = "header-request-live.svg";
pub const HEADER_REVIEW: &str = "header-review.svg";
pub const HEADER_SECURITY_DARK: &str = "header-security-dark.svg";
pub const HEADER_STILL_LOOKING: &str = "header-still-looking.svg";
pub const HEADER_VIEWING: &str = "header-viewing.svg";
pub const HEADER_WELCOME: &str = "header-welcome.svg";

pub fn header_asset_url(file_name: &str) -> String {
    // Email clients are far more reliable with raster assets than inline SVG/webp.
    // Force PNG delivery so Gmail/Outlook preserve the header visuals consistently.
    format!("{HEADER_BASE_URL}/f_png,q_auto,w_1120/{file_name}")
}

pub fn kyc_header_asset(status: &str) -> String {
    match status.trim().to_lowercase().as_str() {
        "approved" | "verified" => header_asset_url(HEADER_CONFIRMED),
        "rejected" => header_asset_url(HEADER_REVIEW),
        _ => header_asset_url(HEADER_REVIEW),
    }
}

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

    // Helper function to build email templates without header images
    fn build_email_template_no_header(
        &self,
        greeting: &str,
        body_text: &str,
        cta_section: Option<&str>,
        footer_note: Option<&str>,
    ) -> String {
        let cta_html = cta_section.unwrap_or("");
        let footer_text = footer_note.unwrap_or("For more information, visit your Verinest dashboard.");

        format!(
            "<!DOCTYPE html>\
<html lang=\"en\">\
<head>\
<meta charset=\"UTF-8\">\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\
<meta name=\"x-apple-disable-message-reformatting\">\
<meta name=\"color-scheme\" content=\"light only\">\
<meta name=\"supported-color-schemes\" content=\"light only\">\
<title>Verinest</title>\
<!--[if mso]>\
<style type=\"text/css\">\
body, table, td, a, p {{ font-family: Arial, sans-serif !important; }}\
</style>\
<![endif]-->\
</head>\
<body style=\"margin:0;padding:0;background-color:#E8E2DA;background:#E8E2DA;-webkit-text-size-adjust:100%;-ms-text-size-adjust:100%;\">\
<center style=\"width:100%;background-color:#E8E2DA;background:#E8E2DA;\">\
<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\" width=\"100%\" style=\"background-color:#E8E2DA;background:#E8E2DA;mso-table-lspace:0pt;mso-table-rspace:0pt;\">\
<tr>\
<td align=\"center\" style=\"padding:24px 12px;\">\
<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\" width=\"560\" style=\"width:560px;max-width:560px;background-color:#FFFFFF;background:#FFFFFF;border-radius:20px;overflow:hidden;\" bgcolor=\"#FFFFFF\">\
<tr>\
<td style=\"padding:36px 40px 32px 40px;background-color:#FFFFFF;background:#FFFFFF;\" bgcolor=\"#FFFFFF\">\
<p style=\"margin:0 0 14px 0;font-size:15px;line-height:1.5;font-weight:600;color:#1A1814;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">{0}</p>\
<p style=\"margin:0 0 20px 0;font-size:13.5px;line-height:1.75;color:#5A5248;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">{1}</p>\
{2}\
<div style=\"height:1px;line-height:1px;font-size:1px;background:#EDE8E0;margin:28px 0;\">&nbsp;</div>\
<p style=\"margin:0;font-size:12px;line-height:1.75;color:#9A8F84;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">{3}</p>\
</td>\
</tr>\
<tr>\
<td style=\"padding:28px 40px;background-color:#161412;background:#161412;border-top:1px solid #2A2520;\" bgcolor=\"#161412\">\
<p style=\"margin:0 0 16px 0;font-size:11px;line-height:1.7;color:#8C8277;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">Verinest helps seekers, agents, and landlords connect through verified listings, clearer pricing, and faster rental matching across Nigeria.</p>\
<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\" width=\"100%\" style=\"margin:0 0 16px 0;\">\
<tr>\
<td style=\"padding:0 18px 0 0;\"><a href=\"https://verinest.ng\" style=\"font-size:10px;line-height:1.4;color:#B89E89;text-decoration:none;letter-spacing:0.06em;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">Browse Homes</a></td>\
<td style=\"padding:0 18px 0 0;\"><a href=\"https://verinest.ng/post\" style=\"font-size:10px;line-height:1.4;color:#B89E89;text-decoration:none;letter-spacing:0.06em;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">Post a Need</a></td>\
<td style=\"padding:0 18px 0 0;\"><a href=\"https://verinest.ng/agents\" style=\"font-size:10px;line-height:1.4;color:#B89E89;text-decoration:none;letter-spacing:0.06em;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">For Agents</a></td>\
<td style=\"padding:0;\"><a href=\"https://verinest.ng/help\" style=\"font-size:10px;line-height:1.4;color:#B89E89;text-decoration:none;letter-spacing:0.06em;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">Help Centre</a></td>\
</tr>\
</table>\
<div style=\"height:1px;line-height:1px;font-size:1px;background:#2A2520;margin:0 0 14px 0;\">&nbsp;</div>\
<p style=\"margin:0;font-size:10px;line-height:1.6;color:#6D6157;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">© 2026 Verinest. All rights reserved. · Nigeria<br>You are receiving this because you have an account at verinest.ng</p>\
</td>\
</tr>\
</table>\
</td>\
</tr>\
</table>\
</center>\
</body>\
</html>",
            greeting, body_text, cta_html, footer_text,
        )
    }

    // Helper function to build refined email templates with header SVGs
    fn build_email_template(
        &self,
        header_image_url: &str,
        greeting: &str,
        body_text: &str,
        cta_section: Option<&str>,
        footer_note: Option<&str>,
    ) -> String {
        let cta_html = cta_section.unwrap_or("");
        let footer_text = footer_note.unwrap_or("For more information, visit your Verinest dashboard.");

        format!(
            "<!DOCTYPE html>\
<html lang=\"en\">\
<head>\
<meta charset=\"UTF-8\">\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\
<meta name=\"x-apple-disable-message-reformatting\">\
<meta name=\"color-scheme\" content=\"light only\">\
<meta name=\"supported-color-schemes\" content=\"light only\">\
<title>Verinest</title>\
<!--[if mso]>\
<style type=\"text/css\">\
body, table, td, a, p {{ font-family: Arial, sans-serif !important; }}\
</style>\
<![endif]-->\
</head>\
<body style=\"margin:0;padding:0;background-color:#E8E2DA;background:#E8E2DA;-webkit-text-size-adjust:100%;-ms-text-size-adjust:100%;\">\
<center style=\"width:100%;background-color:#E8E2DA;background:#E8E2DA;\">\
<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\" width=\"100%\" style=\"background-color:#E8E2DA;background:#E8E2DA;mso-table-lspace:0pt;mso-table-rspace:0pt;\">\
<tr>\
<td align=\"center\" style=\"padding:24px 12px;\">\
<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\" width=\"560\" style=\"width:560px;max-width:560px;background-color:#FFFFFF;background:#FFFFFF;border-radius:20px;overflow:hidden;\" bgcolor=\"#FFFFFF\">\
<tr>\
<td style=\"padding:0;\">\
<img src=\"{0}\" alt=\"Verinest Email Header\" width=\"560\" style=\"display:block;width:100%;max-width:560px;height:auto;border:0;outline:none;text-decoration:none;\" />\
</td>\
</tr>\
<tr>\
<td style=\"padding:36px 40px 32px 40px;background-color:#FFFFFF;background:#FFFFFF;\" bgcolor=\"#FFFFFF\">\
<p style=\"margin:0 0 14px 0;font-size:15px;line-height:1.5;font-weight:600;color:#1A1814;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">{1}</p>\
<p style=\"margin:0 0 20px 0;font-size:13.5px;line-height:1.75;color:#5A5248;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">{2}</p>\
{3}\
<div style=\"height:1px;line-height:1px;font-size:1px;background:#EDE8E0;margin:28px 0;\">&nbsp;</div>\
<p style=\"margin:0;font-size:12px;line-height:1.75;color:#9A8F84;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">{4}</p>\
</td>\
</tr>\
<tr>\
<td style=\"padding:28px 40px;background-color:#161412;background:#161412;border-top:1px solid #2A2520;\" bgcolor=\"#161412\">\
<p style=\"margin:0 0 16px 0;font-size:11px;line-height:1.7;color:#8C8277;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">Verinest helps seekers, agents, and landlords connect through verified listings, clearer pricing, and faster rental matching across Nigeria.</p>\
<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\" width=\"100%\" style=\"margin:0 0 16px 0;\">\
<tr>\
<td style=\"padding:0 18px 0 0;\"><a href=\"https://verinest.ng\" style=\"font-size:10px;line-height:1.4;color:#B89E89;text-decoration:none;letter-spacing:0.06em;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">Browse Homes</a></td>\
<td style=\"padding:0 18px 0 0;\"><a href=\"https://verinest.ng/post\" style=\"font-size:10px;line-height:1.4;color:#B89E89;text-decoration:none;letter-spacing:0.06em;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">Post a Need</a></td>\
<td style=\"padding:0 18px 0 0;\"><a href=\"https://verinest.ng/agents\" style=\"font-size:10px;line-height:1.4;color:#B89E89;text-decoration:none;letter-spacing:0.06em;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">For Agents</a></td>\
<td style=\"padding:0;\"><a href=\"https://verinest.ng/help\" style=\"font-size:10px;line-height:1.4;color:#B89E89;text-decoration:none;letter-spacing:0.06em;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">Help Centre</a></td>\
</tr>\
</table>\
<div style=\"height:1px;line-height:1px;font-size:1px;background:#2A2520;margin:0 0 14px 0;\">&nbsp;</div>\
<p style=\"margin:0;font-size:10px;line-height:1.6;color:#6D6157;font-family:Arial,'Helvetica Neue',Helvetica,sans-serif;\">© 2026 Verinest. All rights reserved. · Nigeria<br>You are receiving this because you have an account at verinest.ng</p>\
</td>\
</tr>\
</table>\
</td>\
</tr>\
</table>\
</center>\
</body>\
</html>",
            header_image_url, greeting, body_text, cta_html, footer_text,
        )
    }

    pub fn verification_email(
        &self,
        to: String,
        full_name: &str,
        verification_link: &str,
        header_image_url: &str,
    ) -> OutboundEmail {
        let cta_html = format!(
            r#"<div style="margin:28px 0;"><a href="{link}" style="display:inline-block;background:#C4714A;color:#FFFFFF;text-decoration:none;font-size:13px;font-weight:600;padding:15px 32px;border-radius:12px;width:100%;text-align:center;box-sizing:border-box;">Verify My Email →</a></div>"#,
            link = verification_link
        );

        let footer_note = format!("If the button doesn't work, copy and paste this link in your browser: {}", verification_link);
        
        let html = self.build_email_template(
            header_image_url,
            &format!("Hi {},", full_name),
            "Your account has been created successfully. To activate it fully, confirm your email address using the secure link below.",
            Some(&cta_html),
            Some(&footer_note),
        );

        OutboundEmail {
            to,
            subject: "Verify your Verinest email".to_string(),
            text: format!(
                "Hello {}, verify your email by opening this link: {}", full_name, verification_link
            ),
            html,
        }
    }

    pub fn welcome_email(&self, to: String, full_name: &str, action_url: &str, header_image_url: &str) -> OutboundEmail {
        let cta_html = format!(
            r#"<div style="margin:28px 0;"><a href="{url}" style="display:inline-block;background:#C4714A;color:#FFFFFF;text-decoration:none;font-size:13px;font-weight:600;padding:15px 32px;border-radius:12px;width:100%;text-align:center;box-sizing:border-box;">Post Your First Need →</a></div>"#,
            url = action_url
        );
        
        let html = self.build_email_template(
            header_image_url,
            &format!("Hi {},", full_name),
            "You've just joined a platform built to make renting in Nigeria less risky, less stressful, and much faster. Post what you need, receive responses from verified providers, and manage everything from one place.",
            Some(&cta_html),
            Some("If you have any questions getting started, reply to this email directly. A real person will respond — not a bot."),
        );

        OutboundEmail {
            to,
            subject: "Welcome to Verinest".to_string(),
            text: format!(
                "Hi {}, welcome to Verinest. Open your dashboard to get started: {}", full_name, action_url
            ),
            html,
        }
    }

    pub fn verification_code_email(
        &self,
        to: String,
        full_name: &str,
        code: &str,
        header_image_url: &str,
    ) -> OutboundEmail {
        let cta_html = format!(
            r#"<div style="background:#FAF7F3;border-radius:16px;padding:32px;text-align:center;margin:28px 0;border:1px dashed #E0D8CE;"><p style="font-size:9px;letter-spacing:0.25em;text-transform:uppercase;color:#9A8F84;margin-bottom:16px;">Your verification code</p><div style="font-family:'Cormorant Garamond',serif;font-size:56px;font-weight:600;color:#C4714A;letter-spacing:0.2em;line-height:1;margin-bottom:12px;word-break:break-all;">{code}</div><p style="font-size:11px;color:#B0A090;">Expires in 10 minutes · Do not share this code</p></div><div style="background:#FFF3ED;border-left:3px solid #C4714A;padding:14px 18px;border-radius:0 10px 10px 0;margin-bottom:20px;"><p style="font-size:12px;color:#7A4020;line-height:1.6;">⚠️ <strong>Verinest will never ask for this code by phone or WhatsApp.</strong> If someone is asking for it, this is a scam attempt. Do not share it.</p></div>"#,
            code = code
        );

        let html = self.build_email_template(
            header_image_url,
            &format!("Hi {},", full_name),
            "You requested a verification code for your Verinest account. Enter this code in the app to continue.",
            Some(&cta_html),
            Some("If you didn't request this code, you can safely ignore this email. Your account remains secure."),
        );

        OutboundEmail {
            to,
            subject: "Your Verinest verification code".to_string(),
            text: format!("Hello {}, your Verinest verification code is {}. Do not share this code. Expires in 10 minutes.", full_name, code),
            html,
        }
    }

    pub fn password_reset_email(
        &self,
        to: String,
        full_name: &str,
        reset_link: &str,
    ) -> OutboundEmail {
        let cta_html = format!(
            r#"<div style="margin:28px 0;"><a href="{link}" style="display:inline-block;background:#C4714A;color:#FFFFFF;text-decoration:none;font-size:13px;font-weight:600;padding:15px 32px;border-radius:12px;width:100%;text-align:center;box-sizing:border-box;">Reset My Password →</a></div>"#,
            link = reset_link
        );

        let footer_note = format!("If the button doesn't work, copy and paste this link in your browser: {}", reset_link);
        
        let html = self.build_email_template_no_header(
            &format!("Hi {},", full_name),
            "Someone requested a password reset for your Verinest account. If this was you, click the link below to create a new password. This link expires in 1 hour.",
            Some(&cta_html),
            Some(&footer_note),
        );

        OutboundEmail {
            to,
            subject: "Reset your Verinest password".to_string(),
            text: format!(
                "Hello {}, click this link to reset your password: {}\n\nThis link expires in 1 hour.\n\nIf you didn't request this, you can safely ignore this email.",
                full_name, reset_link
            ),
            html,
        }
    }

    pub fn kyc_status_email(
        &self,
        to: String,
        full_name: &str,
        verification_status: &str,
        notes: Option<&str>,
        header_image_url: &str,
    ) -> OutboundEmail {
        let notes_text = notes.unwrap_or("Your verification process has been completed.");
        let normalized_status = verification_status.to_lowercase();
        let is_approved = normalized_status == "verified" || normalized_status == "approved";
        
        let info_card = if is_approved {
            format!(r#"<div style="background:#EBF7EF;border-left:4px solid #3A8A52;padding:16px 18px;border-radius:0 10px 10px 0;margin:18px 0;"><div style="font-size:13px;font-weight:600;color:#3A8A52;margin-bottom:6px;">Your verified badge is active</div><div style="font-size:13px;color:#5A5248;line-height:1.7;">Your profile now carries the trust signal that helps others decide who to work with on Verinest.</div></div>"#)
        } else {
            format!(r#"<div style="background:#FFF3ED;border-left:4px solid #C4714A;padding:16px 18px;border-radius:0 10px 10px 0;margin:18px 0;"><div style="font-size:13px;font-weight:600;color:#C4714A;margin-bottom:6px;">Attention Required</div><div style="font-size:13px;color:#7A4020;line-height:1.7;">{}</div></div>"#, notes_text)
        };
        
        let status_message = if is_approved {
            "Congratulations — your Verinest verification has been reviewed and approved."
        } else {
            "Your Verinest verification status update requires your attention. Please review the details below."
        };

        let html = self.build_email_template(
            header_image_url,
            &format!("Hi {},", full_name),
            status_message,
            Some(&info_card),
            Some(&notes_text),
        );

        OutboundEmail {
            to,
            subject: if is_approved {
                "Your Verinest verification is approved".to_string()
            } else {
                "Your Verinest verification needs attention".to_string()
            },
            text: format!(
                "Hello {}, your Verinest verification status has been updated to: {}. {}", 
                full_name, verification_status, notes_text
            ),
            html,
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
        header_image_url: &str,
    ) -> OutboundEmail {
        let role_label = if for_provider {
            "Schedule Reminder"
        } else {
            "Upcoming Visit"
        };
        let _title = if for_provider {
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
        let badge = role_label;
        let intro = body_intro.clone();
        let footer_note = "Please be ready a few minutes early and keep communication inside Verinest for your safety.";
        let overdue = false;

        OutboundEmail {
            to,
            subject,
            text: format!(
                "Hi {recipient_name}, {body_intro} Time: {schedule_label}. Location: {property_location}. Open: {action_url}"
            ),
            html: render_schedule_email_html(
                badge,
                recipient_name,
                &intro,
                property_title,
                property_location,
                booking_type,
                &schedule_label,
                action_copy,
                action_url,
                footer_note,
                overdue,
                header_image_url,
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
        header_image_url: &str,
    ) -> OutboundEmail {
        let role_label = if for_provider {
            "Past Schedule"
        } else {
            "Missed Schedule"
        };
        let _title = if for_provider {
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
        let badge = role_label;
        let intro = body_intro.clone();
        let footer_note = "If the meeting happened, update your status in Verinest. If it did not, reschedule inside the platform so both sides stay aligned.";
        let overdue = true;

        OutboundEmail {
            to,
            subject,
            text: format!(
                "Hi {recipient_name}, {body_intro} Original time: {schedule_label}. Location: {property_location}. Open: {action_url}"
            ),
            html: render_schedule_email_html(
                badge,
                recipient_name,
                &intro,
                property_title,
                property_location,
                booking_type,
                &schedule_label,
                action_copy,
                action_url,
                footer_note,
                overdue,
                header_image_url,
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
        header_image_url: &str,
    ) -> OutboundEmail {
        OutboundEmail {
            to,
            subject: "A verified agent responded to your need".to_string(),
            text: format!(
                "Hi {full_name}, {agent_name} responded to your need for {property_title}. View the listing: {action_url}"
            ),
            html: format!(
                "<!DOCTYPE html>\n<html lang=\"en\" style=\"margin:0;padding:0;\">\n  <body style=\"margin:0;padding:20px;background:#E8E2DA;font-family:'DM Sans',Arial,sans-serif;\">\n    <div style=\"max-width:560px;margin:0 auto;background:#ffffff;border-radius:16px;overflow:hidden;box-shadow:0 4px 20px rgba(0,0,0,0.08);\">\n      <div style=\"background:#F9F6F1;padding:24px 32px;border-bottom:1px solid #EEE8E1;display:flex;align-items:center;justify-content:space-between;\">\n        <div style=\"display:flex;align-items:center;gap:12px;\">\n          <div style=\"width:40px;height:40px;background:#C4714A;border-radius:8px;display:flex;align-items:center;justify-content:center;flex-shrink:0;\">\n            <svg width=\"22\" height=\"22\" viewBox=\"0 0 24 24\" fill=\"none\">\n              <path d=\"M3 11L12 3l9 8\" stroke=\"white\" stroke-width=\"2.2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>\n              <rect x=\"9\" y=\"13\" width=\"6\" height=\"8\" rx=\"3\" fill=\"white\"/>\n            </svg>\n          </div>\n          <div style=\"font-family:'Cormorant Garamond',serif;font-size:20px;font-weight:600;color:#1A1814;letter-spacing:-0.3px;\">Veri<span style=\"font-style:italic;color:#C4714A;\">nest</span></div>\n        </div>\n        <div style=\"font-size:8px;letter-spacing:0.15em;text-transform:uppercase;color:#C4714A;background:#FEF1E8;padding:6px 12px;border-radius:20px;font-weight:500;\">New Match</div>\n      </div>\n      <img src=\"{header_image_url}\" alt=\"Email Header\" style=\"width:100%;height:auto;display:block;max-width:560px;\" />\n      <div style=\"padding:32px;\">\n        <p style=\"font-size:14px;font-weight:500;color:#1A1814;margin:0 0 4px;\">Hi {full_name},</p>\n        <p style=\"font-size:13px;color:#5A5248;line-height:1.75;margin:0 0 20px;\">Good news — a verified agent responded to your need post for a <strong>{property_title}</strong>. Here's what they sent:</p>\n        <div style=\"background:#FAF7F3;border-radius:12px;padding:18px;margin:20px 0;border:1px solid #EEE8E1;\">\n          <div style=\"display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:14px;\">\n            <h3 style=\"font-family:'Cormorant Garamond',serif;font-size:20px;font-weight:600;margin:0;color:#1A1814;flex:1;\">Property Name</h3>\n            <span style=\"background:#C4714A;color:white;font-size:8px;padding:4px 8px;border-radius:4px;text-transform:uppercase;font-weight:600;\">Verified Listing</span>\n          </div>\n          <p style=\"font-size:13px;color:#9A8F84;margin:0 0 10px;\">Location Details</p>\n          <p style=\"font-size:14px;font-weight:600;color:#1A1814;margin:0 0 12px;\">{property_price}</p>\n          <p style=\"font-size:12px;color:#5A5248;margin:0;\">3 bed • 2 bath</p>\n        </div>\n        <div style=\"background:#FAF7F3;border-radius:12px;padding:16px;margin:20px 0;display:flex;gap:12px;border:1px solid #EEE8E1;\">\n          <div style=\"width:40px;height:40px;background:#C4714A;border-radius:50%;flex-shrink:0;display:flex;align-items:center;justify-content:center;\">\n            <span style=\"color:white;font-weight:600;font-size:12px;\">AG</span>\n          </div>\n          <div style=\"flex:1;\">\n            <p style=\"font-size:13px;font-weight:600;color:#1A1814;margin:0 0 2px;\">{agent_name}</p>\n            <p style=\"font-size:12px;color:#9A8F84;margin:0;\">{agent_title}</p>\n            <p style=\"font-size:11px;color:#C4714A;margin:4px 0 0;font-weight:500;\">● VERIFIED PROVIDER</p>\n          </div>\n        </div>\n        <div style=\"margin:24px 0;text-align:center;\">\n          <a href=\"{action_url}\" style=\"display:inline-block;background:#C4714A;color:#ffffff;text-decoration:none;font-size:13px;font-weight:600;padding:12px 28px;border-radius:8px;letter-spacing:0.04em;margin-right:10px;\">View Full Listing →</a>\n          <a href=\"{action_url}\" style=\"display:inline-block;background:transparent;color:#C4714A;text-decoration:none;font-size:13px;font-weight:600;padding:12px 28px;border:1.5px solid #C4714A;border-radius:8px;letter-spacing:0.04em;\">Schedule Viewing</a>\n        </div>\n        <div style=\"height:1px;background:#EEE8E1;margin:20px 0;\"></div>\n        <p style=\"font-size:11px;color:#9A8F84;line-height:1.7;margin:0;\">This agent is verified on Verinest. All communication and viewings should happen through the platform — do not send money or sign anything outside of Verinest.</p>\n      </div>\n      <div style=\"padding:24px 32px 30px;background:#1A1A1A;border-top:1px solid #EEE8E1;\">\n        <div style=\"font-family:'Cormorant Garamond',serif;font-size:16px;font-weight:600;color:#ffffff;margin-bottom:8px;\">Veri<span style=\"font-style:italic;color:#C4714A;\">nest</span></div>\n        <p style=\"font-size:11px;line-height:1.7;color:rgba(255,255,255,0.65);margin:0;\">Matches expire after 48 hours if not reviewed. Log in to your Verinest dashboard to see all your active matches.</p>\n        <div style=\"display:flex;gap:16px;margin-top:12px;\">\n          <a href=\"#\" style=\"font-size:11px;color:#C4714A;text-decoration:none;\">My Dashboard</a>\n          <a href=\"#\" style=\"font-size:11px;color:#C4714A;text-decoration:none;\">My Matches</a>\n          <a href=\"#\" style=\"font-size:11px;color:#C4714A;text-decoration:none;\">Help Centre</a>\n          <a href=\"#\" style=\"font-size:11px;color:#C4714A;text-decoration:none;\">Unsubscribe</a>\n        </div>\n        <p style=\"font-size:10px;color:rgba(255,255,255,0.45);margin:12px 0 0;border-top:1px solid rgba(255,255,255,0.1);padding-top:12px;\">© 2026 Verinest. All rights reserved. Nigeria</p>\n      </div>\n    </div>\n  </body>\n</html>",
                header_image_url = header_image_url,
                full_name = full_name,
                property_title = property_title,
                property_price = property_price,
                agent_name = agent_name,
                agent_title = agent_title,
                action_url = action_url
            ),
        }
    }

    pub fn offer_received_email(&self, to: String, full_name: &str, provider_name: &str, property_title: &str, request_title: &str, action_url: &str, header_image_url: &str) -> OutboundEmail {
        OutboundEmail {
            to,
            subject: format!("New offer from {} for {}", provider_name, property_title),
            text: format!(
                "Hi {full_name}, you received an offer from {provider_name} for {property_title} on your request \"{request_title}\". Open the link to view: {action_url}"
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
                .hero-image {{ width: 100%; height: auto; display: block; }}
                .content {{ padding: 36px 32px; font-size: 13px; line-height: 1.75; color: #5A5248; }}
                .content p {{ margin: 0 0 16px; }}
                .content p.greeting {{ font-size: 14px; font-weight: bold; color: #1A1814; margin-bottom: 12px; }}
                .offer-details {{ background: #FAF7F3; padding: 16px; border-radius: 8px; margin: 20px 0; border-left: 4px solid #C4714A; }}
                .offer-details p {{ margin: 0 0 8px; font-size: 12px; }}
                .offer-details p:last-child {{ margin-bottom: 0; }}
                .offer-details .label {{ color: #9A8F84; font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em; }}
                .offer-details .value {{ color: #1A1814; font-weight: bold; font-size: 13px; }}
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
                <div class="badge">New Offer</div>
                </div>
                <img src="{header_image_url}" alt="Offer Header" style="width:100%;height:auto;display:block;max-width:580px;" />
                <div class="content">
                <p class="greeting">Hi {full_name},</p>
                <p><strong>{provider_name}</strong> sent you an offer for <strong>{property_title}</strong> on your request <strong>"{request_title}"</strong>.</p>
                <div class="offer-details">
                <p><span class="label">Property</span><br><span class="value">{property_title}</span></p>
                <p style="margin-top:8px;"><span class="label">From</span><br><span class="value">{provider_name}</span></p>
                </div>
                <p>Review the full offer details and respond to secure your next home.</p>
                <div style="margin:24px 0;"><a href="{action_url}" style="display:inline-block;background:#C4714A;color:#FFFFFF;text-decoration:none;font-size:13px;font-weight:bold;padding:14px 28px;border-radius:8px;">View Offer →</a></div>
                <div style="height:1px;background:#EDE8E0;margin:24px 0;"></div>
                <p style="font-size:12px;color:#9A8F84;">You received this email because you posted a need on Verinest. Manage your offers and preferences in your dashboard.</p>
                </div>
                <div class="footer-section">
                <div class="footer-brand">Veri<em>nest</em></div>
                <p class="footer-text">Verinest helps seekers, agents, and landlords connect through verified listings, clearer pricing, and faster rental matching across Nigeria.</p>
                <p class="copyright">© 2026 Verinest. All rights reserved. Nigeria</p>
                </div>
                </div>
                </body>
                </html>"#
            ),
        }
    }

    pub fn property_moderation_email(
        &self,
        to: String,
        full_name: &str,
        property_title: &str,
        property_location: &str,
        action_taken: &str,
        reason: &str,
        action_url: &str,
        header_image_url: &str,
    ) -> OutboundEmail {
        let subject = format!("Verinest listing update: {action_taken}");
        let text = format!(
            "Hi {full_name}, your listing \"{property_title}\" has been {action_taken}. Reason: {reason}. Location: {property_location}. Open: {action_url}"
        );
        let html = format!(
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
                        <div style="font-size:9px;letter-spacing:0.2em;text-transform:uppercase;color:#B85C38;background:#FFF1EB;padding:5px 10px;border-radius:20px;">Listing review</div>
                    </div>
                    <img src="{header_image_url}" alt="Email Header" style="width:100%;height:auto;display:block;max-width:560px;" />
                    <div style="padding:36px 40px;">
                        <p style="font-size:15px;font-weight:500;color:#1A1814;margin:0 0 14px;">Hi {full_name},</p>
                        <p style="font-size:13.5px;color:#5A5248;line-height:1.75;margin:0 0 18px;">A Verinest admin reviewed one of your listings and applied a moderation action.</p>
                        <div style="background:#FFF8F4;border:1px solid #F0E0D4;border-radius:18px;padding:18px 20px;margin:0 0 20px;">
                            <div style="font-size:10px;letter-spacing:0.2em;text-transform:uppercase;color:#9A8F84;margin-bottom:10px;">Listing details</div>
                            <div style="font-size:17px;font-weight:600;color:#1A1814;margin-bottom:6px;">{property_title}</div>
                            <div style="font-size:12px;color:#9A8F84;line-height:1.7;">
                                <div><strong style="color:#1A1814;">Action:</strong> {action_taken}</div>
                                <div><strong style="color:#1A1814;">Location:</strong> {property_location}</div>
                                <div><strong style="color:#1A1814;">Reason:</strong> {reason}</div>
                            </div>
                        </div>
                        <div style="margin:0 0 20px;">
                            <a href="{action_url}" style="display:inline-block;background:#C4714A;color:#ffffff;text-decoration:none;font-size:12px;font-weight:500;padding:14px 22px;border-radius:999px;letter-spacing:0.04em;">Open listing →</a>
                        </div>
                        <div style="height:1px;background:#EFE9E2;margin:24px 0;"></div>
                        <p style="font-size:12px;color:#9A8F84;line-height:1.7;margin:0;">If you believe this action was applied in error, reply to this email with the listing title and your clarification.</p>
                    </div>
                    <div style="padding:24px 40px 30px;background:#FAF7F3;border-top:1px solid #EDE8E0;">
                        <div style="font-family:'Cormorant Garamond',serif;font-size:18px;font-weight:600;color:#1A1814;margin-bottom:10px;">Veri<span style="font-style:italic;color:#C4714A;">nest</span></div>
                        <p style="font-size:11px;line-height:1.7;color:#9A8F84;margin:0;">This is an automated listing moderation notice from Verinest.</p>
                    </div>
                </div>
            </body>
            </html>"#,
        );

        OutboundEmail { to, subject, html, text }
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
        header_image_url: &str,
    ) -> String {
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
        <img src="{header_image_url}" alt="Email Header" style="width:100%;height:auto;display:block;max-width:560px;" />
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
            <p style="font-size:11px;line-height:1.7;color:#9A8F84;margin:0;">This is an automated schedule email from Verinest. Please manage visits, confirmations, and reschedules inside of platform.</p>
        </div>
        </div>
    </body>
    </html>"#,
            badge_fg = badge_fg,
            badge_bg = badge_bg,
            header_image_url = header_image_url,
            badge = badge,
            recipient_name = recipient_name,
            intro = intro,
            card_bg = card_bg,
            property_title = property_title,
            booking_type = booking_type,
            schedule_label = schedule_label,
            action_url = action_url,
            action_copy = action_copy,
            footer_note = footer_note,
            property_location = property_location,
        )
    }

    // Include enhanced email templates
