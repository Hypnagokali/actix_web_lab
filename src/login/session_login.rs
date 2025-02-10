use std::sync::Arc;

use actix_web::{Error, web::{Data, Json}, HttpRequest, HttpResponse, Responder};
use serde::Serialize;

use super::{LoadUserService, LoginToken};

pub struct SessionLoginHandler<T: LoadUserService, U>
{
    user_service: Arc<T>,
    mfa_condition: Arc<Option<fn(&U, &HttpRequest) -> bool>>,
}

impl<T, U> SessionLoginHandler<T, U>
where
    T: LoadUserService
{
    pub fn new(user_service: T) -> Self {
        Self {
            user_service: Arc::new(user_service),
            mfa_condition: Arc::new(None),
        }
    }

    pub fn with_mfa_condition(user_service: T, mfa_condition: fn(&U, &HttpRequest) -> bool) -> Self {
        Self {
            user_service: Arc::new(user_service),
            mfa_condition: Arc::new(Some(mfa_condition)),
        }
    }
}

async fn login<T: LoadUserService<User = U>, U> (
    login_token: Json<LoginToken>,
    user_service: Data<Arc<T>>,
    mfa_condition: Data<Arc<Option<fn(&U, &HttpRequest) -> bool>>>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    match user_service
        .load_user(&login_token.username, &login_token.password)
        .await
    {
        Ok(u) => {
            user_service.on_success_handler(&req, &u).await?;
            if let Some(condition) = mfa_condition.as_ref().as_ref() {
                if (condition)(&u, &req) {
                    println!("Multifactor required");
                } else {
                    println!("Multifactor is NOT required");
                }
            }

            Ok(HttpResponse::Ok())
        }
        Err(e) => {
            user_service.on_error_handler(&req).await?;
            Err(e.into())
        }
    }
}

impl<T, U> ::actix_web::dev::HttpServiceFactory for SessionLoginHandler<T, U>
where 
    T: LoadUserService<User=U> + 'static,
    U: 'static,
{
    fn register(self, __config: &mut actix_web::dev::AppService) {        
        let __resource = ::actix_web::Resource::new("/login")
            .name("login")
            .guard(::actix_web::guard::Post())
            .app_data(Data::new(Arc::clone(&self.user_service)))
            .app_data(Data::new(Arc::clone(&self.mfa_condition)))
            .to(login::<T, U>);
        ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
    }
}
