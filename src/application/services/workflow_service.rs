use crate::{
    domain::{
        properties::{PropertyDetail, PropertyRepository, PropertyStatus},
        responses::ResponseRepository,
        users::{User, UserRepository, UserRole},
        workflow::{
            AssignPropertyAgentInput, CertifySiteVisitInput, CreateLiveVideoSessionInput,
            CreatePropertyAgentRequestInput, CreateSiteVisitInput, CreateThreadMessageInput, LiveVideoSession,
            PropertyAgentRequest, RequestThreadView, SiteVisitView, UpdateLiveVideoSessionInput,
            UpdateSiteVisitInput, WorkflowRepository,
        },
    },
    infrastructure::{cache::CacheService, livekit::LiveKitService},
    interfaces::http::errors::AppError,
    utils::validation,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct WorkflowService {
    workflow: WorkflowRepository,
    responses: ResponseRepository,
    properties: PropertyRepository,
    users: UserRepository,
    livekit: LiveKitService,
    cache: CacheService,
}

impl WorkflowService {
    pub fn new(
        workflow: WorkflowRepository,
        responses: ResponseRepository,
        properties: PropertyRepository,
        users: UserRepository,
        livekit: LiveKitService,
        cache: CacheService,
    ) -> Self {
        Self {
            workflow,
            responses,
            properties,
            users,
            livekit,
            cache,
        }
    }

    pub async fn add_thread_message(
        &self,
        actor: &User,
        response_id: Uuid,
        input: CreateThreadMessageInput,
    ) -> Result<RequestThreadView, AppError> {
        validation::validate_required(&input.message, "message")?;
        let context = self.load_response_context(response_id).await?;
        self.ensure_response_participant(actor.id, &context)?;
        let thread = self.workflow.get_or_create_thread(&context).await?;
        self.workflow
            .add_thread_message(thread.id, actor.id, input.message.trim())
            .await?;
        self.workflow
            .get_thread_view(response_id)
            .await?
            .ok_or_else(|| AppError::internal("request thread could not be loaded"))
    }

    pub async fn get_thread(&self, actor: &User, response_id: Uuid) -> Result<RequestThreadView, AppError> {
        let context = self.load_response_context(response_id).await?;
        self.ensure_response_participant(actor.id, &context)?;
        self.workflow
            .get_thread_view(response_id)
            .await?
            .ok_or_else(|| AppError::not_found("request thread not found"))
    }

    pub async fn create_live_video_session(
        &self,
        actor: &User,
        response_id: Uuid,
        input: CreateLiveVideoSessionInput,
    ) -> Result<LiveVideoSession, AppError> {
        let context = self.load_response_context(response_id).await?;
        if actor.id != context.buyer_id {
            return Err(AppError::forbidden("only the buyer can request a live video session"));
        }
        let room_name = self.livekit.room_name_for_session(Uuid::new_v4());
        let session = self
            .workflow
            .create_live_video_session(
                &context,
                actor.id,
                &room_name,
                input.scheduled_at,
                input.tracking_notes.as_deref().map(str::trim),
            )
            .await?;
        Ok(session)
    }

    pub async fn update_live_video_session(
        &self,
        actor: &User,
        session_id: Uuid,
        input: UpdateLiveVideoSessionInput,
    ) -> Result<LiveVideoSession, AppError> {
        if let Some(status) = input.status.as_deref() {
            self.validate_live_video_status(status)?;
        }
        let existing = self
            .workflow
            .find_live_video_session(session_id)
            .await?
            .ok_or_else(|| AppError::not_found("live video session not found"))?;
        if actor.id != existing.buyer_id && actor.id != existing.agent_id {
            return Err(AppError::forbidden("you cannot update this live video session"));
        }

        self.workflow
            .update_live_video_session(
                session_id,
                input.status.as_deref(),
                input.scheduled_at,
                input.started_at,
                input.ended_at,
                input.tracking_notes.as_deref().map(str::trim),
            )
            .await?
            .ok_or_else(|| AppError::internal("live video session could not be updated"))
    }

    pub async fn get_live_video_session_access(
        &self,
        actor: &User,
        session_id: Uuid,
    ) -> Result<crate::domain::workflow::LiveVideoSessionAccess, AppError> {
        let session = self
            .workflow
            .find_live_video_session(session_id)
            .await?
            .ok_or_else(|| AppError::not_found("live video session not found"))?;
        if actor.id != session.buyer_id && actor.id != session.agent_id {
            return Err(AppError::forbidden("you cannot access this live video session"));
        }

        let participant_identity = format!("{}:{}", actor.role_label(), actor.id);
        let participant_name = actor.full_name.clone();
        let metadata = serde_json::json!({
            "user_id": actor.id,
            "role": actor.role_label(),
            "session_id": session.id,
            "response_id": session.response_id
        })
        .to_string();
        let token = self.livekit.create_join_token(
            &session.room_name,
            &participant_identity,
            &participant_name,
            &metadata,
            true,
        )?;

        Ok(crate::domain::workflow::LiveVideoSessionAccess {
            server_url: self.livekit.server_url().to_string(),
            room_name: session.room_name.clone(),
            participant_identity,
            participant_name,
            token,
            session,
        })
    }

    pub async fn create_site_visit(
        &self,
        actor: &User,
        response_id: Uuid,
        input: CreateSiteVisitInput,
    ) -> Result<SiteVisitView, AppError> {
        validation::validate_required(&input.meeting_point, "meeting_point")?;
        let context = self.load_response_context(response_id).await?;
        if actor.id != context.buyer_id {
            return Err(AppError::forbidden("only the buyer can schedule a site visit"));
        }
        if !self
            .workflow
            .is_property_linked_to_response(response_id, input.property_id)
            .await?
        {
            return Err(AppError::bad_request("property is not linked to this response"));
        }

        let site_visit = self
            .workflow
            .create_site_visit(&context, input.property_id, input.scheduled_at, input.meeting_point.trim())
            .await?;
        self.workflow
            .find_site_visit_view(site_visit.id)
            .await?
            .ok_or_else(|| AppError::internal("site visit could not be loaded"))
    }

    pub async fn update_site_visit(
        &self,
        actor: &User,
        site_visit_id: Uuid,
        input: UpdateSiteVisitInput,
    ) -> Result<SiteVisitView, AppError> {
        if let Some(status) = input.status.as_deref() {
            self.validate_site_visit_status(status)?;
        }
        if let Some(meeting_point) = input.meeting_point.as_deref() {
            validation::validate_required(meeting_point, "meeting_point")?;
        }

        let existing = self
            .workflow
            .find_site_visit_view(site_visit_id)
            .await?
            .ok_or_else(|| AppError::not_found("site visit not found"))?;
        if actor.id != existing.site_visit.buyer_id && actor.id != existing.site_visit.agent_id {
            return Err(AppError::forbidden("you cannot update this site visit"));
        }

        let updated = self
            .workflow
            .update_site_visit(
                site_visit_id,
                input.scheduled_at,
                input.meeting_point.as_deref().map(str::trim),
                input.status.as_deref(),
            )
            .await?
            .ok_or_else(|| AppError::internal("site visit could not be updated"))?;

        self.workflow
            .find_site_visit_view(updated.id)
            .await?
            .ok_or_else(|| AppError::internal("site visit could not be loaded"))
    }

    pub async fn certify_site_visit(
        &self,
        actor: &User,
        site_visit_id: Uuid,
        input: CertifySiteVisitInput,
    ) -> Result<SiteVisitView, AppError> {
        validation::validate_required(&input.notes, "notes")?;
        let existing = self
            .workflow
            .find_site_visit_view(site_visit_id)
            .await?
            .ok_or_else(|| AppError::not_found("site visit not found"))?;
        if actor.id != existing.site_visit.buyer_id && actor.id != existing.site_visit.agent_id {
            return Err(AppError::forbidden("you cannot certify this site visit"));
        }

        self.workflow
            .certify_site_visit(site_visit_id, actor.id, input.notes.trim())
            .await?
            .ok_or_else(|| AppError::internal("site visit could not be certified"))?;
        self.workflow
            .find_site_visit_view(site_visit_id)
            .await?
            .ok_or_else(|| AppError::internal("site visit could not be loaded"))
    }

    pub async fn create_property_agent_request(
        &self,
        actor: &User,
        property_id: Uuid,
        input: CreatePropertyAgentRequestInput,
    ) -> Result<PropertyAgentRequest, AppError> {
        if actor.role != UserRole::Landlord {
            return Err(AppError::forbidden("only landlords can request an agent"));
        }
        let property = self
            .properties
            .find_detail_by_id_including_unpublished(property_id)
            .await?
            .ok_or_else(|| AppError::not_found("property not found"))?;
        if property.owner_id != actor.id {
            return Err(AppError::forbidden("you can only request an agent for your own property"));
        }
        if property.self_managed {
            return Err(AppError::bad_request("self-managed properties cannot request an agent"));
        }
        if let Some(agent_id) = input.requested_agent_id {
            self.users
                .find_agent_by_id(agent_id)
                .await?
                .ok_or_else(|| AppError::bad_request("requested agent does not exist"))?;
        }

        self.workflow
            .create_property_agent_request(
                property_id,
                actor.id,
                input.requested_agent_id,
                input.notes.as_deref().map(str::trim),
            )
            .await
            .map_err(Into::into)
    }

    pub async fn assign_property_agent(
        &self,
        actor: &User,
        property_id: Uuid,
        input: AssignPropertyAgentInput,
    ) -> Result<PropertyDetail, AppError> {
        if actor.role != UserRole::Landlord {
            return Err(AppError::forbidden("only landlords can assign an agent"));
        }
        let property = self
            .properties
            .find_detail_by_id_including_unpublished(property_id)
            .await?
            .ok_or_else(|| AppError::not_found("property not found"))?;
        if property.owner_id != actor.id {
            return Err(AppError::forbidden("you can only assign agents to your own property"));
        }
        if property.status != PropertyStatus::Verified && property.status != PropertyStatus::Published {
            return Err(AppError::bad_request("property must be verified before assigning an agent"));
        }
        self.users
            .find_agent_by_id(input.agent_id)
            .await?
            .ok_or_else(|| AppError::bad_request("assigned agent does not exist"))?;

        self.properties
            .assign_agent(property_id, input.agent_id)
            .await?
            .ok_or_else(|| AppError::not_found("property not found"))?;
        self.workflow.fulfill_property_agent_request(property_id).await?;
        self.invalidate_property_cache().await?;

        self.properties
            .find_detail_by_id_including_unpublished(property_id)
            .await?
            .ok_or_else(|| AppError::internal("property could not be loaded"))
    }

    pub async fn verify_property(&self, actor: &User, property_id: Uuid) -> Result<PropertyDetail, AppError> {
        if actor.role != UserRole::Agent {
            return Err(AppError::forbidden("only agents can verify properties"));
        }
        let property = self
            .properties
            .find_detail_by_id_including_unpublished(property_id)
            .await?
            .ok_or_else(|| AppError::not_found("property not found"))?;
        if property.status == PropertyStatus::Published {
            return Err(AppError::bad_request("published properties are already verified"));
        }

        self.properties
            .verify_property(property_id, actor.id)
            .await?
            .ok_or_else(|| AppError::not_found("property not found"))?;
        self.invalidate_property_cache().await?;
        self.properties
            .find_detail_by_id_including_unpublished(property_id)
            .await?
            .ok_or_else(|| AppError::internal("property could not be loaded"))
    }

    pub async fn publish_property(&self, actor: &User, property_id: Uuid) -> Result<PropertyDetail, AppError> {
        let property = self
            .properties
            .find_detail_by_id_including_unpublished(property_id)
            .await?
            .ok_or_else(|| AppError::not_found("property not found"))?;
        if actor.id != property.owner_id && Some(actor.id) != property.agent_id {
            return Err(AppError::forbidden("you cannot publish this property"));
        }
        if property.status != PropertyStatus::Verified {
            return Err(AppError::bad_request("property must be verified before publishing"));
        }

        self.properties
            .publish_property(property_id)
            .await?
            .ok_or_else(|| AppError::bad_request("property could not be published"))?;
        self.invalidate_property_cache().await?;
        self.properties
            .find_detail_by_id_including_unpublished(property_id)
            .await?
            .ok_or_else(|| AppError::internal("property could not be loaded"))
    }

    fn ensure_response_participant(
        &self,
        actor_id: Uuid,
        context: &crate::domain::workflow::ResponseWorkflowContext,
    ) -> Result<(), AppError> {
        if actor_id != context.buyer_id && actor_id != context.agent_id {
            return Err(AppError::forbidden("you do not have access to this response workflow"));
        }
        Ok(())
    }

    async fn load_response_context(
        &self,
        response_id: Uuid,
    ) -> Result<crate::domain::workflow::ResponseWorkflowContext, AppError> {
        let context = self
            .workflow
            .response_context(response_id)
            .await?
            .ok_or_else(|| AppError::not_found("response not found"))?;
        let response = self
            .responses
            .find_context(response_id)
            .await?
            .ok_or_else(|| AppError::not_found("response not found"))?;
        if response.post_author_id != context.buyer_id || response.responder_id != context.agent_id {
            return Err(AppError::internal("response workflow context is inconsistent"));
        }
        Ok(context)
    }

    async fn invalidate_property_cache(&self) -> Result<(), AppError> {
        self.cache.invalidate_namespace("properties:list").await?;
        self.cache.invalidate_namespace("properties:detail").await?;
        Ok(())
    }

    fn validate_live_video_status(&self, status: &str) -> Result<(), AppError> {
        let valid = ["requested", "scheduled", "live", "completed", "cancelled"];
        if valid.contains(&status) {
            return Ok(());
        }
        Err(AppError::bad_request("invalid live video status"))
    }

    fn validate_site_visit_status(&self, status: &str) -> Result<(), AppError> {
        let valid = ["scheduled", "confirmed", "completed", "cancelled", "certified"];
        if valid.contains(&status) {
            return Ok(());
        }
        Err(AppError::bad_request("invalid site visit status"))
    }
}
