use anyhow::Result;
use chrono::{DateTime, Timelike, Utc};
use sqlx::{FromRow, PgPool};
use tokio::time::{Duration, sleep};
use tracing::{error, info};
use uuid::Uuid;

use crate::infrastructure::email::MailService;

#[derive(Clone)]
pub struct BookingEmailReminderService {
    pool: PgPool,
    mail_service: MailService,
    app_base_url: String,
}

#[derive(Debug, Clone, FromRow)]
struct BookingReminderRow {
    id: Uuid,
    booking_type: String,
    scheduled_for: DateTime<Utc>,
    property_title: String,
    property_location: String,
    seeker_name: String,
    seeker_email: String,
    provider_name: String,
    provider_email: String,
    provider_role: String,
    seeker_upcoming_email_sent_at: Option<DateTime<Utc>>,
    provider_upcoming_email_sent_at: Option<DateTime<Utc>>,
    seeker_past_due_email_sent_at: Option<DateTime<Utc>>,
    provider_past_due_email_sent_at: Option<DateTime<Utc>>,
}

impl BookingEmailReminderService {
    pub fn new(pool: PgPool, mail_service: MailService, app_base_url: String) -> Self {
        Self {
            pool,
            mail_service,
            app_base_url: app_base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn spawn_cron(self) {
        tokio::spawn(async move {
            sleep(duration_until_next_minute()).await;

            loop {
                if let Err(error) = self.run_once().await {
                    error!("booking reminder cron failed: {error:#}");
                }
                sleep(duration_until_next_minute()).await;
            }
        });
    }

    pub async fn run_once(&self) -> Result<()> {
        self.send_upcoming_reminders().await?;
        self.send_past_due_reminders().await?;
        Ok(())
    }

    async fn send_upcoming_reminders(&self) -> Result<()> {
        let bookings = sqlx::query_as::<_, BookingReminderRow>(
            r#"
            SELECT
                b.id,
                b.booking_type,
                b.scheduled_for,
                COALESCE(p.title, 'Scheduled property') AS property_title,
                COALESCE(p.location, 'Nigeria') AS property_location,
                seeker.full_name AS seeker_name,
                seeker.email AS seeker_email,
                provider.full_name AS provider_name,
                provider.email AS provider_email,
                provider.role::text AS provider_role,
                b.seeker_upcoming_email_sent_at,
                b.provider_upcoming_email_sent_at,
                b.seeker_past_due_email_sent_at,
                b.provider_past_due_email_sent_at
            FROM bookings b
            INNER JOIN properties p ON p.id = b.property_id
            INNER JOIN users seeker ON seeker.id = b.seeker_user_id
            INNER JOIN users provider ON provider.id = b.provider_user_id
            WHERE b.status IN ('pending', 'confirmed')
              AND b.scheduled_for > NOW()
              AND b.scheduled_for <= NOW() + INTERVAL '1 hour'
              AND (
                    b.seeker_upcoming_email_sent_at IS NULL
                 OR b.provider_upcoming_email_sent_at IS NULL
              )
            ORDER BY b.scheduled_for ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for booking in bookings {
            self.send_upcoming_for_booking(&booking).await?;
        }

        Ok(())
    }

    async fn send_past_due_reminders(&self) -> Result<()> {
        let bookings = sqlx::query_as::<_, BookingReminderRow>(
            r#"
            SELECT
                b.id,
                b.booking_type,
                b.scheduled_for,
                COALESCE(p.title, 'Scheduled property') AS property_title,
                COALESCE(p.location, 'Nigeria') AS property_location,
                seeker.full_name AS seeker_name,
                seeker.email AS seeker_email,
                provider.full_name AS provider_name,
                provider.email AS provider_email,
                provider.role::text AS provider_role,
                b.seeker_upcoming_email_sent_at,
                b.provider_upcoming_email_sent_at,
                b.seeker_past_due_email_sent_at,
                b.provider_past_due_email_sent_at
            FROM bookings b
            INNER JOIN properties p ON p.id = b.property_id
            INNER JOIN users seeker ON seeker.id = b.seeker_user_id
            INNER JOIN users provider ON provider.id = b.provider_user_id
            WHERE b.status IN ('pending', 'confirmed')
              AND b.scheduled_for <= NOW() - INTERVAL '1 hour'
              AND (
                    b.seeker_past_due_email_sent_at IS NULL
                 OR b.provider_past_due_email_sent_at IS NULL
              )
            ORDER BY b.scheduled_for ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for booking in bookings {
            self.send_past_due_for_booking(&booking).await?;
        }

        Ok(())
    }

    async fn send_upcoming_for_booking(&self, booking: &BookingReminderRow) -> Result<()> {
        let seeker_url = format!("{}/seeker/bookings", self.app_base_url);
        if booking.seeker_upcoming_email_sent_at.is_none() {
            let email = self.mail_service.schedule_upcoming_email(
                booking.seeker_email.clone(),
                &booking.seeker_name,
                &booking.provider_name,
                &booking.property_title,
                &booking.property_location,
                &booking.booking_type,
                booking.scheduled_for,
                &seeker_url,
                false,
                "https://res.cloudinary.com/dui0hakkq/image/upload/v1715020800/verinest/headers/header-viewing.svg",
            );
            self.mail_service.send(email).await?;
            sqlx::query("UPDATE bookings SET seeker_upcoming_email_sent_at = NOW() WHERE id = $1")
                .bind(booking.id)
                .execute(&self.pool)
                .await?;
        }

        let provider_url = provider_schedule_url(&self.app_base_url, &booking.provider_role);
        if booking.provider_upcoming_email_sent_at.is_none() {
            let email = self.mail_service.schedule_upcoming_email(
                booking.provider_email.clone(),
                &booking.provider_name,
                &booking.seeker_name,
                &booking.property_title,
                &booking.property_location,
                &booking.booking_type,
                booking.scheduled_for,
                &provider_url,
                true,
                "https://res.cloudinary.com/dui0hakkq/image/upload/v1715020800/verinest/headers/header-viewing.svg",
            );
            self.mail_service.send(email).await?;
            sqlx::query(
                "UPDATE bookings SET provider_upcoming_email_sent_at = NOW() WHERE id = $1",
            )
            .bind(booking.id)
            .execute(&self.pool)
            .await?;
        }

        info!(
            "sent upcoming booking reminder emails for booking {}",
            booking.id
        );
        Ok(())
    }

    async fn send_past_due_for_booking(&self, booking: &BookingReminderRow) -> Result<()> {
        let seeker_url = format!("{}/seeker/bookings", self.app_base_url);
        if booking.seeker_past_due_email_sent_at.is_none() {
            let email = self.mail_service.schedule_past_due_email(
                booking.seeker_email.clone(),
                &booking.seeker_name,
                &booking.provider_name,
                &booking.property_title,
                &booking.property_location,
                &booking.booking_type,
                booking.scheduled_for,
                &seeker_url,
                false,
                "https://res.cloudinary.com/dui0hakkq/image/upload/v1715020800/verinest/headers/header-post-viewing.svg",
            );
            self.mail_service.send(email).await?;
            sqlx::query("UPDATE bookings SET seeker_past_due_email_sent_at = NOW() WHERE id = $1")
                .bind(booking.id)
                .execute(&self.pool)
                .await?;
        }

        let provider_url = provider_schedule_url(&self.app_base_url, &booking.provider_role);
        if booking.provider_past_due_email_sent_at.is_none() {
            let email = self.mail_service.schedule_past_due_email(
                booking.provider_email.clone(),
                &booking.provider_name,
                &booking.seeker_name,
                &booking.property_title,
                &booking.property_location,
                &booking.booking_type,
                booking.scheduled_for,
                &provider_url,
                true,
                "https://res.cloudinary.com/dui0hakkq/image/upload/v1715020800/verinest/headers/header-post-viewing.svg",
            );
            self.mail_service.send(email).await?;
            sqlx::query(
                "UPDATE bookings SET provider_past_due_email_sent_at = NOW() WHERE id = $1",
            )
            .bind(booking.id)
            .execute(&self.pool)
            .await?;
        }

        info!(
            "sent past-due booking reminder emails for booking {}",
            booking.id
        );
        Ok(())
    }
}

fn duration_until_next_minute() -> Duration {
    let now = Utc::now();
    let seconds = 60_u64.saturating_sub(u64::from(now.second()));
    let nanos = u64::from(now.nanosecond());
    let extra = if nanos == 0 { 0 } else { 1 };
    Duration::from_secs(seconds + extra)
}

fn provider_schedule_url(app_base_url: &str, provider_role: &str) -> String {
    if provider_role.eq_ignore_ascii_case("landlord") {
        format!("{app_base_url}/landlord/calendar")
    } else {
        format!("{app_base_url}/provider/calendar")
    }
}
