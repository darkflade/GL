use crate::application::contracts::NewUser;
use crate::application::ports::UserRepository;
use crate::domain::model::RepoError;
use crate::web::error::AppError;
use crate::web::handlers::utils::{map_repo_error, parse_uuid};
use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
) -> Result<HttpResponse, AppError> {
    let user_id = match user {
        Some(u) => u.id().map_err(|err| {
            log::warn!("failed to resolve identity id from session: {err}");
            AppError::unauthorized("Unauthorized")
        })?,
        None => return Err(AppError::unauthorized("Unauthorized")),
    };

    let uuid = parse_uuid(&user_id, "user id")?;

    let user_model = user_repo
        .find_by_id(uuid)
        .await
        .map_err(|err| map_repo_error(err, "User not found", "users.find_by_id"))?;

    Ok(HttpResponse::Ok().json(UserProfile {
        id: user_model.id,
        username: user_model.username,
    }))
}

pub async fn login_user<UR: UserRepository + Clone>(
    form: web::Json<AuthRequest>,
    user_repo: web::Data<UR>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user = user_repo
        .find_by_username(&form.username)
        .await
        .map_err(|_| AppError::unauthorized("Invalid username or password"))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|err| AppError::internal(format!("users.login invalid password hash: {err}")))?;

    let is_valid = Argon2::default()
        .verify_password(form.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_valid {
        return Err(AppError::unauthorized("Invalid username or password"));
    }

    Identity::login(&req.extensions(), user.id.to_string()).map_err(|err| {
        AppError::internal(format!(
            "users.login failed to set identity in session: {err}"
        ))
    })?;

    Ok(HttpResponse::Ok().json("Successfully logged in"))
}

pub async fn logout_user(user: Identity) -> Result<HttpResponse, AppError> {
    user.logout();
    Ok(HttpResponse::Ok().json("Successfully logged out"))
}

pub async fn register_user<UR: UserRepository + Clone>(
    form: web::Json<AuthRequest>,
    user_repo: web::Data<UR>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(form.password.as_bytes(), &salt)
        .map_err(|err| AppError::bad_request(format!("Invalid password: {err}")))?
        .to_string();

    let new_user = NewUser {
        username: form.username.clone(),
        password_hash,
    };

    let user_id = user_repo.create(new_user).await.map_err(|err| match err {
        RepoError::StorageError => AppError::conflict("User already exists"),
        RepoError::NotFound => AppError::internal("users.register impossible not found state"),
    })?;

    Identity::login(&req.extensions(), user_id.to_string()).map_err(|err| {
        AppError::internal(format!(
            "users.register failed to set identity in session: {err}"
        ))
    })?;

    Ok(HttpResponse::Created().body("User registered"))
}
