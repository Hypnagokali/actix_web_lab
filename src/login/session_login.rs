use std::sync::Arc;

use actix_web::{Error, web::{Data, Json}, HttpRequest, HttpResponse, Responder};

use super::{LoadUserService, LoginToken};

pub struct SessionLoginHandler<T: LoadUserService> {
    user_service: Arc<T>
}

impl<T> SessionLoginHandler<T> 
where
    T: LoadUserService
{
    pub fn new(user_service: T) -> Self {
        Self {
            user_service: Arc::new(user_service),
        }
    }

}


async fn login<T: LoadUserService>(
    login_token: Json<LoginToken>,
    user_service: Data<Arc<T>>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    match user_service
        .load_user(&login_token.username, &login_token.password)
        .await
    {
        Ok(u) => {
            user_service.on_success_handler(&req, &u).await?;
            Ok(HttpResponse::Ok())
        }
        Err(e) => {
            user_service.on_error_handler(&req).await?;
            Err(e.into())
        }
    }
}

impl<T> ::actix_web::dev::HttpServiceFactory for SessionLoginHandler<T>
where 
    T: LoadUserService + 'static
{
    fn register(self, __config: &mut actix_web::dev::AppService) {        
        let __resource = ::actix_web::Resource::new("/login")
            .name("login")
            .guard(::actix_web::guard::Post())
            .app_data(Data::new(Arc::clone(&self.user_service)))
            .to(login::<T>);
        ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
    }
}
