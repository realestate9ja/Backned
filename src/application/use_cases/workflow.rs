use crate::{
    application::services::WorkflowService,
    domain::{
        properties::PropertyDetail,
        users::User,
        workflow::{
            AssignPropertyAgentInput, CertifySiteVisitInput, CreateLiveVideoSessionInput,
            CreatePropertyAgentRequestInput, CreateSiteVisitInput, CreateThreadMessageInput, LiveVideoSession,
            LiveVideoSessionAccess, PropertyAgentRequest, RequestThreadView, SiteVisitView, UpdateLiveVideoSessionInput,
            UpdateSiteVisitInput,
        },
    },
    interfaces::http::errors::AppError,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct WorkflowUseCases {
    service: WorkflowService,
}

impl WorkflowUseCases {
    pub fn new(service: WorkflowService) -> Self {
        Self { service }
    }

    pub async fn add_thread_message(
        &self,
        actor: &User,
        response_id: Uuid,
        input: CreateThreadMessageInput,
    ) -> Result<RequestThreadView, AppError> {
        self.service.add_thread_message(actor, response_id, input).await
    }

    pub async fn get_thread(&self, actor: &User, response_id: Uuid) -> Result<RequestThreadView, AppError> {
        self.service.get_thread(actor, response_id).await
    }

    pub async fn create_live_video_session(
        &self,
        actor: &User,
        response_id: Uuid,
        input: CreateLiveVideoSessionInput,
    ) -> Result<LiveVideoSession, AppError> {
        self.service.create_live_video_session(actor, response_id, input).await
    }

    pub async fn update_live_video_session(
        &self,
        actor: &User,
        session_id: Uuid,
        input: UpdateLiveVideoSessionInput,
    ) -> Result<LiveVideoSession, AppError> {
        self.service.update_live_video_session(actor, session_id, input).await
    }

    pub async fn get_live_video_session_access(
        &self,
        actor: &User,
        session_id: Uuid,
    ) -> Result<LiveVideoSessionAccess, AppError> {
        self.service.get_live_video_session_access(actor, session_id).await
    }

    pub async fn create_site_visit(
        &self,
        actor: &User,
        response_id: Uuid,
        input: CreateSiteVisitInput,
    ) -> Result<SiteVisitView, AppError> {
        self.service.create_site_visit(actor, response_id, input).await
    }

    pub async fn update_site_visit(
        &self,
        actor: &User,
        site_visit_id: Uuid,
        input: UpdateSiteVisitInput,
    ) -> Result<SiteVisitView, AppError> {
        self.service.update_site_visit(actor, site_visit_id, input).await
    }

    pub async fn certify_site_visit(
        &self,
        actor: &User,
        site_visit_id: Uuid,
        input: CertifySiteVisitInput,
    ) -> Result<SiteVisitView, AppError> {
        self.service.certify_site_visit(actor, site_visit_id, input).await
    }

    pub async fn create_property_agent_request(
        &self,
        actor: &User,
        property_id: Uuid,
        input: CreatePropertyAgentRequestInput,
    ) -> Result<PropertyAgentRequest, AppError> {
        self.service.create_property_agent_request(actor, property_id, input).await
    }

    pub async fn assign_property_agent(
        &self,
        actor: &User,
        property_id: Uuid,
        input: AssignPropertyAgentInput,
    ) -> Result<PropertyDetail, AppError> {
        self.service.assign_property_agent(actor, property_id, input).await
    }

    pub async fn verify_property(&self, actor: &User, property_id: Uuid) -> Result<PropertyDetail, AppError> {
        self.service.verify_property(actor, property_id).await
    }

    pub async fn publish_property(&self, actor: &User, property_id: Uuid) -> Result<PropertyDetail, AppError> {
        self.service.publish_property(actor, property_id).await
    }
}
