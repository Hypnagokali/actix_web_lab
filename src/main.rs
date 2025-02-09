
use std::{future::ready, marker::PhantomData, ops::Deref, sync::Arc};

use actix_web::{post, web::{Data, Json}, App, Error, FromRequest, Handler, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError};
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub trait LoadUserService: Send + Sync {
    type User;

    fn load_user(&self, username: &str, password: &str) -> LocalBoxFuture<'_, Result<Self::User, LoadUserError>>;
    fn on_success_handler(&self, req: &HttpRequest, user: &Self::User) -> LocalBoxFuture<'_, Result<(), LoadUserError>>;
    fn on_error_handler(&self, req: &HttpRequest) -> LocalBoxFuture<'_, Result<(), LoadUserError>>;
}

#[allow(dead_code)]
pub struct MyUser {
    pub name: String,
}

pub struct HardCodedLoadUserService {}

impl LoadUserService for HardCodedLoadUserService {
    type User = MyUser;

    fn load_user(&self, username: &str, password: &str) -> LocalBoxFuture<'_, Result<Self::User, LoadUserError>> {
        if username == "test" && password == "test123" {
            Box::pin(ready(Ok(
                MyUser {
                        name: "Dummy".to_owned(),
                }
            )))
        } else {
            Box::pin(async {
                Err(LoadUserError::LoginFailed)
            })
        }
    }
    
    fn on_success_handler(&self, _req: &HttpRequest, _user: &Self::User) -> LocalBoxFuture<'_, Result<(), LoadUserError>> {
        println!("Login successful");
        Box::pin(ready(Ok(())))
    }
    
    fn on_error_handler(&self, _req: &HttpRequest) -> LocalBoxFuture<'_, Result<(), LoadUserError>> {
        println!("Login failed");
        Box::pin(ready(Ok(())))
    }
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum LoadUserError {
    #[error("Username or password wrong")]
    LoginFailed,
    #[error("Error in an handler function: {0}")]
    HandlerError(String),
}

impl ResponseError for LoadUserError {
    fn error_response(&self) -> HttpResponse {      
        match self {
            LoadUserError::LoginFailed => {
                println!("Return 401 Error");
                HttpResponse::Unauthorized().body(self.to_string())
            },
            LoadUserError::HandlerError(_) => HttpResponse::InternalServerError().body(self.to_string()),
        }
    }
}

#[derive(Serialize,Deserialize)]
pub struct LoginToken {
    pub username: String,
    pub password: String,
}


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

impl<T> ::actix_web::dev::HttpServiceFactory for SessionLoginHandler<T>
where 
    T: LoadUserService + 'static
{
    fn register(self, __config: &mut actix_web::dev::AppService) {

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
        
        let __resource = ::actix_web::Resource::new("/login")
            .name("login")
            .guard(::actix_web::guard::Post())
            .app_data(Data::new(Arc::clone(&self.user_service)))
            .to(login::<T>);
        ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()>{
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .service(SessionLoginHandler::new(HardCodedLoadUserService {}))

    }).bind(("127.0.0.1", 8080))?
    .run()
    .await
}
