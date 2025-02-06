use std::sync::Arc;

use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};

trait LoadUserService: Send + Sync {
    type User;

    fn load_user(&self, username: &str, password: &str) -> Self::User;
}

struct MyUser {
    pub name: String,
}

struct HardCodedLoadUserService {}

impl LoadUserService for HardCodedLoadUserService {
    type User = MyUser;

    fn load_user(&self, _username: &str, _password: &str) -> Self::User {
        MyUser {
            name: "Dummy".to_owned(),
        }
    }
}


#[get("/login")]
async fn login(_user_service: Data<Arc<dyn LoadUserService<User = MyUser>>>) -> impl Responder {
    HttpResponse::Ok()
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
