use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    properties::PropertyListItem,
    workflow::{
        LiveVideoSession, PropertyAgentRequest, RequestThread, RequestThreadView, ResponseWorkflowContext,
        SiteVisit, SiteVisitCertification, SiteVisitView, ThreadMessage,
    },
};

#[derive(Clone)]
pub struct WorkflowRepository {
    pool: PgPool,
}

impl WorkflowRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn response_context(&self, response_id: Uuid) -> Result<Option<ResponseWorkflowContext>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            response_id: Uuid,
            post_id: Uuid,
            buyer_id: Uuid,
            agent_id: Uuid,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                r.id AS response_id,
                r.post_id,
                p.author_id AS buyer_id,
                r.responder_id AS agent_id
            FROM responses r
            INNER JOIN posts p ON p.id = r.post_id
            WHERE r.id = $1
            "#,
        )
        .bind(response_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| ResponseWorkflowContext {
            response_id: row.response_id,
            post_id: row.post_id,
            buyer_id: row.buyer_id,
            agent_id: row.agent_id,
        }))
    }

    pub async fn get_or_create_thread(&self, context: &ResponseWorkflowContext) -> Result<RequestThread> {
        if let Some(thread) = self.find_thread_by_response_id(context.response_id).await? {
            return Ok(thread);
        }

        let thread = sqlx::query_as::<_, RequestThread>(
            r#"
            INSERT INTO request_threads (
                id, response_id, post_id, buyer_id, agent_id, status, last_message_at
            )
            VALUES ($1, $2, $3, $4, $5, 'open', NOW())
            RETURNING id, response_id, post_id, buyer_id, agent_id, status, last_message_at, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(context.response_id)
        .bind(context.post_id)
        .bind(context.buyer_id)
        .bind(context.agent_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(thread)
    }

    pub async fn find_thread_by_response_id(&self, response_id: Uuid) -> Result<Option<RequestThread>> {
        let thread = sqlx::query_as::<_, RequestThread>(
            r#"
            SELECT id, response_id, post_id, buyer_id, agent_id, status, last_message_at, created_at, updated_at
            FROM request_threads
            WHERE response_id = $1
            "#,
        )
        .bind(response_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(thread)
    }

    pub async fn add_thread_message(&self, thread_id: Uuid, sender_id: Uuid, message: &str) -> Result<ThreadMessage> {
        let message_id = Uuid::new_v4();

        sqlx::query(
            r#"
            UPDATE request_threads
            SET last_message_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(thread_id)
        .execute(&self.pool)
        .await?;

        let item = sqlx::query_as::<_, ThreadMessage>(
            r#"
            INSERT INTO thread_messages (id, thread_id, sender_id, message)
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                thread_id,
                sender_id,
                '' AS sender_name,
                '' AS sender_role,
                message,
                created_at
            "#,
        )
        .bind(message_id)
        .bind(thread_id)
        .bind(sender_id)
        .bind(message)
        .fetch_one(&self.pool)
        .await?;

        let messages = self.list_thread_messages(thread_id).await?;
        let created = messages
            .into_iter()
            .find(|current| current.id == message_id)
            .unwrap_or(item);

        Ok(created)
    }

    pub async fn get_thread_view(&self, response_id: Uuid) -> Result<Option<RequestThreadView>> {
        let Some(thread) = self.find_thread_by_response_id(response_id).await? else {
            return Ok(None);
        };
        let messages = self.list_thread_messages(thread.id).await?;
        Ok(Some(RequestThreadView { thread, messages }))
    }

    pub async fn list_threads_for_user(&self, user_id: Uuid, limit: i64) -> Result<Vec<RequestThread>> {
        let items = sqlx::query_as::<_, RequestThread>(
            r#"
            SELECT id, response_id, post_id, buyer_id, agent_id, status, last_message_at, created_at, updated_at
            FROM request_threads
            WHERE buyer_id = $1 OR agent_id = $1
            ORDER BY last_message_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn create_live_video_session(
        &self,
        context: &ResponseWorkflowContext,
        requested_by_user_id: Uuid,
        room_name: &str,
        scheduled_at: Option<DateTime<Utc>>,
        tracking_notes: Option<&str>,
    ) -> Result<LiveVideoSession> {
        let session_id = Uuid::new_v4();
        let item = sqlx::query_as::<_, LiveVideoSession>(
            r#"
            INSERT INTO live_video_sessions (
                id, response_id, requested_by_user_id, buyer_id, agent_id, provider, room_name, status, scheduled_at, tracking_notes
            )
            VALUES ($1, $2, $3, $4, $5, 'livekit', $6, 'requested', $7, $8)
            RETURNING
                id, response_id, requested_by_user_id, buyer_id, agent_id, provider, room_name, status, scheduled_at,
                started_at, ended_at, tracking_notes, recording_saved, created_at, updated_at
            "#,
        )
        .bind(session_id)
        .bind(context.response_id)
        .bind(requested_by_user_id)
        .bind(context.buyer_id)
        .bind(context.agent_id)
        .bind(room_name)
        .bind(scheduled_at)
        .bind(tracking_notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn update_live_video_session(
        &self,
        session_id: Uuid,
        status: Option<&str>,
        scheduled_at: Option<DateTime<Utc>>,
        started_at: Option<DateTime<Utc>>,
        ended_at: Option<DateTime<Utc>>,
        tracking_notes: Option<&str>,
    ) -> Result<Option<LiveVideoSession>> {
        let item = sqlx::query_as::<_, LiveVideoSession>(
            r#"
            UPDATE live_video_sessions
            SET status = COALESCE($2, status),
                scheduled_at = COALESCE($3, scheduled_at),
                started_at = COALESCE($4, started_at),
                ended_at = COALESCE($5, ended_at),
                tracking_notes = COALESCE($6, tracking_notes),
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, response_id, requested_by_user_id, buyer_id, agent_id, provider, room_name, status, scheduled_at,
                started_at, ended_at, tracking_notes, recording_saved, created_at, updated_at
            "#,
        )
        .bind(session_id)
        .bind(status)
        .bind(scheduled_at)
        .bind(started_at)
        .bind(ended_at)
        .bind(tracking_notes)
        .fetch_optional(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn find_live_video_session(&self, session_id: Uuid) -> Result<Option<LiveVideoSession>> {
        let item = sqlx::query_as::<_, LiveVideoSession>(
            r#"
            SELECT
                id, response_id, requested_by_user_id, buyer_id, agent_id, provider, room_name, status, scheduled_at,
                started_at, ended_at, tracking_notes, recording_saved, created_at, updated_at
            FROM live_video_sessions
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn list_live_video_sessions_for_user(&self, user_id: Uuid, limit: i64) -> Result<Vec<LiveVideoSession>> {
        let items = sqlx::query_as::<_, LiveVideoSession>(
            r#"
            SELECT
                id, response_id, requested_by_user_id, buyer_id, agent_id, provider, room_name, status, scheduled_at,
                started_at, ended_at, tracking_notes, recording_saved, created_at, updated_at
            FROM live_video_sessions
            WHERE buyer_id = $1 OR agent_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn create_site_visit(
        &self,
        context: &ResponseWorkflowContext,
        property_id: Uuid,
        scheduled_at: DateTime<Utc>,
        meeting_point: &str,
    ) -> Result<SiteVisit> {
        let item = sqlx::query_as::<_, SiteVisit>(
            r#"
            INSERT INTO site_visits (
                id, response_id, buyer_id, agent_id, property_id, scheduled_at, meeting_point, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'scheduled')
            RETURNING
                id, response_id, buyer_id, agent_id, property_id, scheduled_at, meeting_point,
                status, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(context.response_id)
        .bind(context.buyer_id)
        .bind(context.agent_id)
        .bind(property_id)
        .bind(scheduled_at)
        .bind(meeting_point)
        .fetch_one(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn update_site_visit(
        &self,
        site_visit_id: Uuid,
        scheduled_at: Option<DateTime<Utc>>,
        meeting_point: Option<&str>,
        status: Option<&str>,
    ) -> Result<Option<SiteVisit>> {
        let item = sqlx::query_as::<_, SiteVisit>(
            r#"
            UPDATE site_visits
            SET scheduled_at = COALESCE($2, scheduled_at),
                meeting_point = COALESCE($3, meeting_point),
                status = COALESCE($4, status),
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, response_id, buyer_id, agent_id, property_id, scheduled_at, meeting_point,
                status, created_at, updated_at
            "#,
        )
        .bind(site_visit_id)
        .bind(scheduled_at)
        .bind(meeting_point)
        .bind(status)
        .fetch_optional(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn certify_site_visit(
        &self,
        site_visit_id: Uuid,
        certified_by: Uuid,
        notes: &str,
    ) -> Result<Option<SiteVisitCertification>> {
        let mut tx = self.pool.begin().await?;

        let site_visit = sqlx::query_scalar::<_, Uuid>("SELECT id FROM site_visits WHERE id = $1")
            .bind(site_visit_id)
            .fetch_optional(&mut *tx)
            .await?;
        if site_visit.is_none() {
            tx.rollback().await?;
            return Ok(None);
        }

        sqlx::query(
            r#"
            UPDATE site_visits
            SET status = 'certified', updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(site_visit_id)
        .execute(&mut *tx)
        .await?;

        let item = sqlx::query_as::<_, SiteVisitCertification>(
            r#"
            INSERT INTO site_visit_certifications (id, site_visit_id, certified_by, notes)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (site_visit_id) DO UPDATE
            SET certified_by = EXCLUDED.certified_by,
                certified_at = NOW(),
                notes = EXCLUDED.notes
            RETURNING id, site_visit_id, certified_by, certified_at, notes
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(site_visit_id)
        .bind(certified_by)
        .bind(notes)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(Some(item))
    }

    pub async fn find_site_visit_view(&self, site_visit_id: Uuid) -> Result<Option<SiteVisitView>> {
        let site_visit = sqlx::query_as::<_, SiteVisit>(
            r#"
            SELECT
                id, response_id, buyer_id, agent_id, property_id, scheduled_at, meeting_point,
                status, created_at, updated_at
            FROM site_visits
            WHERE id = $1
            "#,
        )
        .bind(site_visit_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(site_visit) = site_visit else {
            return Ok(None);
        };

        let property = self.property_list_items(&[site_visit.property_id]).await?;
        let certification = sqlx::query_as::<_, SiteVisitCertification>(
            r#"
            SELECT id, site_visit_id, certified_by, certified_at, notes
            FROM site_visit_certifications
            WHERE site_visit_id = $1
            "#,
        )
        .bind(site_visit.id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property.into_iter().next().map(|property| SiteVisitView {
            site_visit,
            property,
            certification,
        }))
    }

    pub async fn list_site_visit_views_for_user(&self, user_id: Uuid, limit: i64) -> Result<Vec<SiteVisitView>> {
        let visits = sqlx::query_as::<_, SiteVisit>(
            r#"
            SELECT
                id, response_id, buyer_id, agent_id, property_id, scheduled_at, meeting_point,
                status, created_at, updated_at
            FROM site_visits
            WHERE buyer_id = $1 OR agent_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let property_ids = visits.iter().map(|visit| visit.property_id).collect::<Vec<_>>();
        let properties = self.property_list_items(&property_ids).await?;
        let property_map = properties.into_iter().map(|item| (item.id, item)).collect::<HashMap<_, _>>();

        let visit_ids = visits.iter().map(|visit| visit.id).collect::<Vec<_>>();
        let certifications = if visit_ids.is_empty() {
            Vec::new()
        } else {
            sqlx::query_as::<_, SiteVisitCertification>(
                r#"
                SELECT id, site_visit_id, certified_by, certified_at, notes
                FROM site_visit_certifications
                WHERE site_visit_id = ANY($1)
                "#,
            )
            .bind(&visit_ids)
            .fetch_all(&self.pool)
            .await?
        };
        let certification_map = certifications
            .into_iter()
            .map(|item| (item.site_visit_id, item))
            .collect::<HashMap<_, _>>();

        Ok(visits
            .into_iter()
            .filter_map(|site_visit| {
                property_map.get(&site_visit.property_id).cloned().map(|property| SiteVisitView {
                    certification: certification_map.get(&site_visit.id).cloned(),
                    site_visit,
                    property,
                })
            })
            .collect())
    }

    pub async fn create_property_agent_request(
        &self,
        property_id: Uuid,
        landlord_id: Uuid,
        requested_agent_id: Option<Uuid>,
        notes: Option<&str>,
    ) -> Result<PropertyAgentRequest> {
        let item = sqlx::query_as::<_, PropertyAgentRequest>(
            r#"
            INSERT INTO property_agent_requests (
                id, property_id, landlord_id, requested_agent_id, status, notes
            )
            VALUES ($1, $2, $3, $4, 'pending', $5)
            ON CONFLICT (property_id) DO UPDATE
            SET landlord_id = EXCLUDED.landlord_id,
                requested_agent_id = EXCLUDED.requested_agent_id,
                status = 'pending',
                notes = EXCLUDED.notes,
                updated_at = NOW()
            RETURNING
                id, property_id, landlord_id, requested_agent_id, status, notes, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(property_id)
        .bind(landlord_id)
        .bind(requested_agent_id)
        .bind(notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn find_property_agent_request(&self, property_id: Uuid) -> Result<Option<PropertyAgentRequest>> {
        let item = sqlx::query_as::<_, PropertyAgentRequest>(
            r#"
            SELECT
                id, property_id, landlord_id, requested_agent_id, status, notes, created_at, updated_at
            FROM property_agent_requests
            WHERE property_id = $1
            "#,
        )
        .bind(property_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn fulfill_property_agent_request(&self, property_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE property_agent_requests
            SET status = 'fulfilled', updated_at = NOW()
            WHERE property_id = $1
            "#,
        )
        .bind(property_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn is_property_linked_to_response(&self, response_id: Uuid, property_id: Uuid) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM response_properties
                WHERE response_id = $1 AND property_id = $2
            )
            "#,
        )
        .bind(response_id)
        .bind(property_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    pub async fn has_meaningful_interaction(
        &self,
        response_id: Uuid,
        reviewer_id: Uuid,
        reviewee_id: Uuid,
        property_id: Option<Uuid>,
    ) -> Result<bool> {
        let participant_match = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM responses r
                INNER JOIN posts p ON p.id = r.post_id
                WHERE r.id = $1
                  AND (
                    (p.author_id = $2 AND r.responder_id = $3)
                    OR (p.author_id = $3 AND r.responder_id = $2)
                  )
            )
            "#,
        )
        .bind(response_id)
        .bind(reviewer_id)
        .bind(reviewee_id)
        .fetch_one(&self.pool)
        .await?;
        if !participant_match {
            return Ok(false);
        }

        if let Some(property_id) = property_id {
            let property_match = sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM response_properties
                    WHERE response_id = $1 AND property_id = $2
                )
                "#,
            )
            .bind(response_id)
            .bind(property_id)
            .fetch_one(&self.pool)
            .await?;
            if !property_match {
                return Ok(false);
            }
        }

        let completed_video = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM live_video_sessions
                WHERE response_id = $1 AND status = 'completed'
            )
            "#,
        )
        .bind(response_id)
        .fetch_one(&self.pool)
        .await?;
        if completed_video {
            return Ok(true);
        }

        let certified_visit = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM site_visits sv
                INNER JOIN site_visit_certifications svc ON svc.site_visit_id = sv.id
                WHERE sv.response_id = $1
                  AND ($2::uuid IS NULL OR sv.property_id = $2)
            )
            "#,
        )
        .bind(response_id)
        .bind(property_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(certified_visit)
    }

    async fn list_thread_messages(&self, thread_id: Uuid) -> Result<Vec<ThreadMessage>> {
        let messages = sqlx::query_as::<_, ThreadMessage>(
            r#"
            SELECT
                m.id,
                m.thread_id,
                m.sender_id,
                u.full_name AS sender_name,
                u.role::text AS sender_role,
                m.message,
                m.created_at
            FROM thread_messages m
            INNER JOIN users u ON u.id = m.sender_id
            WHERE m.thread_id = $1
            ORDER BY m.created_at ASC
            "#,
        )
        .bind(thread_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    async fn property_list_items(&self, property_ids: &[Uuid]) -> Result<Vec<PropertyListItem>> {
        if property_ids.is_empty() {
            return Ok(Vec::new());
        }

        let items = sqlx::query_as::<_, PropertyListItem>(
            r#"
            SELECT
                p.id,
                p.title,
                p.price,
                p.location,
                p.description,
                p.images,
                p.is_service_apartment,
                p.status,
                p.self_managed,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at,
                p.verified_at
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            WHERE p.id = ANY($1)
            ORDER BY p.created_at DESC
            "#,
        )
        .bind(property_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }
}
