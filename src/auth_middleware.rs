use actix_session::SessionExt;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    middleware::Next,
    Error,
};

pub async fn require_login(
    request: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let authenticated = matches!(
        request.get_session().get::<i32>("user_id"),
        Ok(Some(_))
    );

    if !authenticated {
        return Err(ErrorUnauthorized("Login diperlukan"));
    }

    next.call(request).await
}