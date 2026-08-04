use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use tokio::time::{interval, Duration};
use tracing::{error, info};

use crate::infrastructure::email::{OutboundEmail, MailService};

#[derive(Clone)]
pub struct VerificationReminderService {
    pool: PgPool,
    mail_service: MailService,
    app_base_url: String,
}

#[derive(Debug, Clone, FromRow)]
struct PendingVerificationRow {
    user_name: String,
    user_email: String,
    user_role: String,
    status: String,
    submitted_at: Option<DateTime<Utc>>,
    notes: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct AdminRecipientRow {
    email: String,
    full_name: String,
}

impl VerificationReminderService {
    pub fn new(pool: PgPool, mail_service: MailService, app_base_url: String) -> Self {
        Self {
            pool,
            mail_service,
            app_base_url: app_base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn spawn_cron(self) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(8 * 60 * 60));
            ticker.tick().await;
            loop {
                if let Err(error) = self.run_once().await {
                    error!("verification reminder cron failed: {error:#}");
                }
                ticker.tick().await;
            }
        });
    }

    pub async fn run_once(&self) -> Result<()> {
        let pending = self.pending_verifications().await?;
        if pending.is_empty() {
            info!("verification reminder cron found no pending verifications");
            return Ok(());
        }

        let admins = self.admin_recipients().await?;
        if admins.is_empty() {
            info!(
                "verification reminder cron found {} pending verifications but no admin recipients",
                pending.len()
            );
            return Ok(());
        }

        for admin in &admins {
            let email = self.admin_alert_email(&admin.full_name, &admin.email, &pending);
            self.mail_service.send(email).await?;
        }

        info!(
            "sent verification reminder alerts for {} pending verifications to {} admins",
            pending.len(),
            admins.len()
        );
        Ok(())
    }

    async fn pending_verifications(&self) -> Result<Vec<PendingVerificationRow>> {
        let rows = sqlx::query_as::<_, PendingVerificationRow>(
            r#"
            SELECT
                v.id,
                u.full_name AS user_name,
                u.email AS user_email,
                u.role::text AS user_role,
                v.status,
                COALESCE(v.submitted_at, v.created_at) AS submitted_at,
                v.notes
            FROM verifications v
            INNER JOIN users u ON u.id = v.user_id
            WHERE v.status IN ('submitted', 'pending', 'in_review')
            ORDER BY COALESCE(v.submitted_at, v.created_at) ASC
            LIMIT 25
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn admin_recipients(&self) -> Result<Vec<AdminRecipientRow>> {
        let rows = sqlx::query_as::<_, AdminRecipientRow>(
            r#"
            SELECT full_name, email
            FROM users
            WHERE role IN ('admin', 'super_admin')
              AND email_verified = TRUE
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    fn admin_alert_email(
        &self,
        admin_name: &str,
        admin_email: &str,
        pending: &[PendingVerificationRow],
    ) -> OutboundEmail {
        let review_url = format!("{}/admin/verifications", self.app_base_url);
        let mut items = String::new();
        for row in pending.iter().take(5) {
            let submitted_at = row
                .submitted_at
                .map(|value| value.format("%-d %b %Y, %-I:%M %p").to_string())
                .unwrap_or_else(|| "Recently".to_string());
            let notes = row.notes.as_deref().unwrap_or("No review notes provided.");
            items.push_str(&format!(
                r#"<div style="padding:14px 0;border-bottom:1px solid #EEE8E1;"><div style="font-size:13px;font-weight:600;color:#1A1814;margin-bottom:4px;">{name}</div><div style="font-size:11px;color:#9A8F84;line-height:1.6;margin-bottom:4px;">{email} · {role} · {status}</div><div style="font-size:11px;color:#5A5248;line-height:1.6;margin-bottom:4px;">Submitted {submitted_at}</div><div style="font-size:11px;color:#7A4020;line-height:1.6;">{notes}</div></div>"#,
                name = row.user_name,
                email = row.user_email,
                role = row.user_role,
                status = row.status,
                submitted_at = submitted_at,
                notes = notes,
            ));
        }

        let body = format!(
            "There are {} verification request(s) waiting for review. Approve or reject them from your admin dashboard.",
            pending.len()
        );
        let cta = format!(
            r#"<div style="background:#FAF7F3;border:1px solid #EEE8E1;border-radius:16px;padding:18px 20px;margin:24px 0;"><div style="font-size:13px;font-weight:600;color:#1A1814;margin-bottom:10px;">Pending verification queue</div>{items}</div><div style="margin:28px 0;text-align:center;"><a href="{review_url}" style="display:inline-block;background:#C4714A;color:#FFFFFF;text-decoration:none;font-size:13px;font-weight:600;padding:15px 28px;border-radius:12px;width:100%;text-align:center;box-sizing:border-box;">Review verifications →</a></div>"#,
            review_url = review_url,
        );
        let footer = format!(
            "You are receiving this because you are an admin on Verinest. Pending verifications remain visible until reviewed."
        );
        let html = self.mail_service.build_email_template_no_header(
            &format!("Hi {},", admin_name),
            &body,
            Some(&cta),
            Some(&footer),
        );

        OutboundEmail {
            to: admin_email.to_string(),
            subject: format!("Verinest: {} verification request(s) need review", pending.len()),
            text: format!(
                "Hi {}, there are {} verification requests waiting for review. Open your admin dashboard: {}",
                admin_name,
                pending.len(),
                review_url
            ),
            html,
        }
    }
}
