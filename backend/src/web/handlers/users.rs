use actix_identity::Identity;
use actix_web::{error, web, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::model::{NewUser, User};
use crate::domain::repository::UserRepository;


#[derive(Serialize)]
pub struct UserProfile {
    id: Uuid,
    username: String,
    // avatar: Options<String>
}

#[derive(Deserialize)]
pub struct AuthRequest {
    username: String,
    password: String,
}

pub async fn get_current_user<UR: UserRepository + Clone>(
    user: Option<Identity>,
    user_repo: web::Data<UR>,
) -> Result<HttpResponse, Error> {
    let user_id = match user {
        Some(u) => u.id().unwrap(),
        None => return Ok(HttpResponse::Unauthorized().finish()),
    };

    let uuid = Uuid::parse_str(&user_id)
        .map_err(|_| error::ErrorBadRequest("Bad ID"))?;

    let user_model = user_repo.find_by_id(uuid).await.map_err(|_| {
        error::ErrorNotFound("User not found")
    })?;
    
    Ok(HttpResponse::Ok().json(UserProfile {
        id: user_model.id,
        username: user_model.username,
    }))
}


pub async fn login_user<UR: UserRepository  + Clone>(
    form: web::Json<AuthRequest>,
    user_repo: web::Data<UR>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let user = user_repo.find_by_username(&form.username).await.map_err(|_| {
        error::ErrorUnauthorized("Invalid username or password")
    })?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| error::ErrorInternalServerError("Hash invalid"))?;

    let is_valid = Argon2::default()
        .verify_password(form.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_valid {
        return Err(error::ErrorUnauthorized("Invalid username or password"));
    }

    Identity::login(&req.extensions(), user.id.to_string())?;

    Ok(HttpResponse::Ok().json("Successfully logged in"))
}

pub async fn logout_user(user: Identity) -> Result<HttpResponse, Error> {
    user.logout();
    Ok(HttpResponse::Ok().json("Successfully logged out"))
}

pub async fn register_user<UR: UserRepository + Clone>(
    form:   web::Json<AuthRequest>,
    user_repo:   web::Data<UR>,
    req:    actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(form.password.as_bytes(), &salt)
        .map_err(|_| error::ErrorBadRequest("Invalid password"))?
        .to_string();

    let new_user = NewUser {
        username: form.username.clone(),
        password_hash,
    };

    let user_id = user_repo.create(new_user)
        .await
        .map_err(|_| error::ErrorBadRequest("User already exists or DB error"))?;

    Identity::login(&req.extensions(), user_id.to_string())?;

    Ok(HttpResponse::Created().body("User registered"))
}