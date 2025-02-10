
use std::future::ready;

use actix_web::{App, HttpRequest, HttpServer};
use futures::future::LocalBoxFuture;
use login::{session_login::SessionLoginHandler, LoadUserError, LoadUserService};

mod login;

#[allow(dead_code)]
pub struct MyUser {
    pub name: String,
}

pub struct HardCodedLoadUserService {}

impl LoadUserService for HardCodedLoadUserService {
    type User = MyUser;

    fn load_user(&self, username: &str, password: &str) -> LocalBoxFuture<'_, Result<Self::User, LoadUserError>> {
        if (username == "test" || username == "test2") && password == "test123" {
            Box::pin(ready(Ok(
                MyUser {
                        name: username.to_owned(),
                }
            )))
        } else {
            Box::pin(ready(Err(LoadUserError::LoginFailed)))
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


fn mfa_condition(user: &MyUser, _: &HttpRequest) -> bool {
    user.name == "test"
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .service(SessionLoginHandler::with_mfa_condition(HardCodedLoadUserService {}, mfa_condition))

    }).bind(("127.0.0.1", 8080))?
    .run()
    .await
}
