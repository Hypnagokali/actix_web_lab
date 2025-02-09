use actix_web::{HttpRequest, HttpResponse, ResponseError};
use futures::future::LocalBoxFuture;
use serde::Deserialize;
use thiserror::Error;

pub mod session_login;

#[derive(Deserialize)]
pub struct LoginToken {
    pub username: String,
    pub password: String,
}

pub trait LoadUserService: Send + Sync {
    type User;

    fn load_user(&self, username: &str, password: &str) -> LocalBoxFuture<'_, Result<Self::User, LoadUserError>>;
    fn on_success_handler(&self, req: &HttpRequest, user: &Self::User) -> LocalBoxFuture<'_, Result<(), LoadUserError>>;
    fn on_error_handler(&self, req: &HttpRequest) -> LocalBoxFuture<'_, Result<(), LoadUserError>>;
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
