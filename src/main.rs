
use std::{future::ready, sync::Arc};

use actix_web::{post, web::{Data, Json}, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError};
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use thiserror::Error;

trait LoadUserService: Send + Sync {
    type User;

    fn load_user(&self, username: &str, password: &str) -> LocalBoxFuture<'_, Result<Self::User, LoadUserError>>;

    fn on_success_handler(&self, req: &HttpRequest, user: &Self::User) -> LocalBoxFuture<'_, Result<(), LoadUserError>>;
    fn on_error_handler(&self, req: &HttpRequest) -> LocalBoxFuture<'_, Result<(), LoadUserError>>;
}

struct MyUser {
    pub name: String,
}

struct HardCodedLoadUserService {}

impl LoadUserService for HardCodedLoadUserService {
    type User = MyUser;

    fn load_user(&self, username: &str, password: &str) -> LocalBoxFuture<'_, Result<Self::User, LoadUserError>> {
        if username == "test" && password == "test123" {
            Box::pin(ready(Ok(
                MyUser {
                        name: "Dummy".to_owned(),
                }
            ))
            )
        } else {
            Box::pin(async {
                Err(LoadUserError::LoginFailed)
            })
        }
        
    }
    
    fn on_success_handler(&self, _req: &HttpRequest, _user: &Self::User) -> LocalBoxFuture<'_, Result<(), LoadUserError>> {
        if true {
            println!("Generate error while in success handler");
            let m = LoadUserError::HandlerError("Something went wrong".to_owned());
            Box::pin(ready(Err(m)))
        } else {
            println!("Login successfully");
            Box::pin(ready(Ok(())))
        }
    }
    
    fn on_error_handler(&self, _req: &HttpRequest) -> LocalBoxFuture<'_, Result<(), LoadUserError>> {
        println!("Login failed");
        Box::pin(ready(Ok(())))
    }
}

#[derive(Error, Debug)]
enum LoadUserError {
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


#[post("/login")]
async fn login(login_token: Json<LoginToken>, user_service: Data<Arc<dyn LoadUserService<User = MyUser>>>, req: HttpRequest) -> Result<impl Responder, Error> {
    match user_service.load_user(&login_token.username, &login_token.password).await {
        Ok(u) => {
            user_service.on_success_handler(&req, &u).await?;
            Ok(HttpResponse::Ok())
        },
        Err(e) => {
            user_service.on_error_handler(&req).await?;
            Err(e.into())
        },
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()>{
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let user_service: Arc<dyn LoadUserService<User = MyUser>> = Arc::new(HardCodedLoadUserService {});

    HttpServer::new(move || {
        App::new()
            .service(login)
            .app_data(Data::new(user_service.clone()))

    }).bind(("127.0.0.1", 8080))?
    .run()
    .await
}
