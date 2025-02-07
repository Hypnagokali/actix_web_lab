
use std::sync::Arc;

use actix_web::{get, web::{Data, Json}, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

// TODO: needs to be async
trait LoadUserService: Send + Sync {
    type User;

    fn load_user(&self, username: &str, password: &str) -> Result<Self::User, ()>;

    fn on_success_handler(&self, req: &HttpRequest, user: &Self::User);
    fn on_error_handler(&self, req: &HttpRequest);
}

struct MyUser {
    pub name: String,
}

struct HardCodedLoadUserService {}

impl LoadUserService for HardCodedLoadUserService {
    type User = MyUser;

    fn load_user(&self, username: &str, password: &str) -> Result<Self::User, ()> {
        if username == "test" && password == "test123" {
            Ok(MyUser {
                name: "Dummy".to_owned(),
            })
        } else {
            Err(())
        }
        
    }
    
    fn on_success_handler(&self, _req: &HttpRequest, _user: &Self::User) {
        println!("Login successfully");
    }
    
    fn on_error_handler(&self, _req: &HttpRequest) {
        println!("Login failed!");
    }
}

#[derive(Serialize,Deserialize)]
pub struct LoginToken {
    pub username: String,
    pub password: String,
}


#[get("/login")]
async fn login(login_token: Json<LoginToken>, user_service: Data<Arc<dyn LoadUserService<User = MyUser>>>, req: HttpRequest) -> impl Responder {
    match user_service.load_user(&login_token.username, &login_token.password) {
        Ok(u) => {
            user_service.on_success_handler(&req, &u);
            HttpResponse::Ok()
        },
        Err(_) => {
            user_service.on_error_handler(&req);
            HttpResponse::BadRequest()
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
