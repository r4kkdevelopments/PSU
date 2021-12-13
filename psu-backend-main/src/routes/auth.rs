use account_services::link_discord;
use rocket::{self, Data};
use rocket_contrib::json::{Json, JsonValue};
use serde::Deserialize;

use crate::modules::account_services;
use crate::MainPGDatabase;

use std::{net::SocketAddr, unimplemented};

use rocket::http::{ContentType, Status};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub captcha: String,
}

#[derive(Deserialize)]
pub struct FinaliseRequest {
    pub resetToken: String,
    pub newPassword: String,
    pub captcha: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordReq {
    pub email: String,
    pub captcha: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    pub captcha: String,
}

#[derive(Deserialize)]
pub struct MeRequest {
    pub token: String,
}

pub struct UserAgent(String);

#[derive(Debug)]
pub enum ApiUserAgentError {
    Missing,
}

impl<'a, 'r> FromRequest<'a, 'r> for UserAgent {
    type Error = ApiUserAgentError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let token = request.headers().get_one("User-Agent");
        match token {
            Some(token) => {
                // check validity
                Outcome::Success(UserAgent(token.to_string()))
            }
            None => Outcome::Failure((Status::Unauthorized, ApiUserAgentError::Missing)),
        }
    }
}

pub mod perm;
pub mod premium;

#[post("/auth/update_avatar", data = "<data>")]
// signature requires the request to have a `Content-Type`
pub fn update_avatar(
    conn: MainPGDatabase,
    cont_type: &ContentType,
    data: Data,
) -> Result<String, JsonValue> {
    // this and the next check can be implemented as a request guard but it seems like just
    // more boilerplate than necessary
    if !cont_type.is_form_data() {
        return Err(json!({
          "success": false,
          "message": "Content-Type not multipart/form-data"
        }));
    }
    let (_, boundary) = cont_type
        .params()
        .find(|&(k, _)| k == "boundary")
        .ok_or_else(|| {
            json!({
              "success": false,
              "message": "`Content-Type: multipart/form-data` boundary param not provided"
            })
        })?;

    match account_services::process_avatar_upload(boundary, data, &conn) {
        Ok(resp) => Ok(resp),
        Err(err) => Err(json!({"success": false, "message": err})),
    }
}

#[post("/auth/get_apikey", format = "json", data = "<request_data>")]
pub fn api_key(conn: MainPGDatabase, request_data: Json<MeRequest>) -> JsonValue {
    match account_services::get_api_key(&request_data.token, conn) {
        Ok(data) => {
            return json!({
              "success": true,
              "api_key": data
            });
        }
        Err(errmessage) => {
            return json!({
              "success": false,
              "message": errmessage
            });
        }
    }
}

#[post("/auth/me", format = "json", data = "<request_data>")]
pub fn me(conn: MainPGDatabase, request_data: Json<MeRequest>) -> JsonValue {
    match account_services::get_user(&request_data.token, &conn) {
        Ok(data) => {
            return json!({
              "success": true,
              "userData": data.get_safe_user(&conn)
            });
        }
        Err(errmessage) => {
            return json!({
              "success": false,
              "message": errmessage
            });
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    token: String,
    email: String,
    first_name: String,
    last_name: String,
}

#[post("/auth/me/update", format = "json", data = "<request_data>")]
pub fn update_profile(
    conn: MainPGDatabase,
    request_data: Json<UpdateProfileRequest>,
) -> Result<JsonValue, JsonValue> {
    let user_id = match account_services::is_authenticated(&request_data.token, &conn) {
        Ok(data) => data,
        Err(_err) => return Err(json!({"success": false, "message": "Authentication Failed."})),
    };

    match account_services::update_profile(
        &user_id,
        &request_data.email,
        &request_data.first_name,
        &request_data.last_name,
        &conn,
    ) {
        Ok(data) => Ok(json!({"success": true, "message": data})),
        Err(err) => Err(json!({"success": false, "message": err})),
    }
}

#[derive(Deserialize)]
pub struct DiscordConnectRequest {
    pub token: String,
    pub discord_code: String,
}

#[post("/auth/connnections/discord", format = "json", data = "<request_data>")]
pub fn discord_connection(
    conn: MainPGDatabase,
    request_data: Json<DiscordConnectRequest>,
) -> Result<JsonValue, JsonValue> {
    let user_id = match account_services::is_authenticated(&request_data.token, &conn) {
        Ok(data) => data,
        Err(err) => {
            return Err(json!({
              "success": false,
              "message": err
            }))
        }
    };

    let discord_user =
        match account_services::link_discord(&user_id, &request_data.discord_code, conn) {
            Ok(data) => data,
            Err(err) => return Err(json!({"success": false, "message": err})),
        };

    return Ok(json!({
      "success": true,
      "user": discord_user
    }));
}

#[post(
    "/auth/connnections/delink_discord",
    format = "json",
    data = "<request_data>"
)]
pub fn discord_delink(
    conn: MainPGDatabase,
    request_data: Json<MeRequest>,
) -> Result<JsonValue, JsonValue> {
    let user_id = match account_services::is_authenticated(&request_data.token, &conn) {
        Ok(data) => data,
        Err(err) => {
            return Err(json!({
              "success": false,
              "message": err
            }))
        }
    };

    match account_services::unlink_discord(&user_id, conn) {
        Ok(data) => data,
        Err(err) => return Err(json!({"success": false, "message": err})),
    };

    return Ok(json!({
      "success": true,
      "message": "SUCCESS"
    }));
}

#[post("/auth/finalise_reset", format = "json", data = "<request_data>")]
pub fn finalise_reset(conn: MainPGDatabase, request_data: Json<FinaliseRequest>) -> JsonValue {
    if !verify_recaptcha(&request_data.captcha) {
        return json!({
          "success": false,
          "message": "Captcha validation failed, please try again."
        });
    };
    
    // Check length
    if request_data.newPassword.len() < 8 {
        return json!({"success": false, "message": "Password needs to be more than 8 characters"});
    }

    // Check password score
    let score_estimate = zxcvbn::zxcvbn(&request_data.newPassword, &[]).unwrap();

    if score_estimate.score() <= 2 {
        match score_estimate.feedback() {
            Some(data) => match data.warning() {
                Some(warning) => {
                    return json!({"success": false, "message": format!("Password is too weak, Warning: {}", warning)})
                }
                None => {
                    return json!({"success": false, "message": format!("Password is too weak, Suggestion: {}", data.suggestions()[0])})
                }
            },
            None => return json!({"success": false, "message": "Password is too weak"}),
        }
    }

    // Now reset the password
    match account_services::finalise_password_reset(
        &request_data.resetToken,
        &request_data.newPassword,
        &conn,
    ) {
        Ok(_data) => return json!({"success": true, "message": "SUCCESS"}),
        Err(err) => return json!({"success": false, "message": err}),
    }
}

#[post("/auth/reset_password", format = "json", data = "<request_data>")]
pub fn reset_password(conn: MainPGDatabase, request_data: Json<ResetPasswordReq>) -> JsonValue {
    if !verify_recaptcha(&request_data.captcha) {
        return json!({
          "success": false,
          "message": "Captcha validation failed, please try again."
        });
    };
    
    match account_services::send_reset_email(&request_data.email, &conn) {
        Ok(_data) => return json!({"success": true, "message": "SUCCESS"}),
        Err(err) => return json!({"success": false, "message": err}),
    }
}

#[post("/auth/regenerate_apikey", format = "json", data = "<request_data>")]
pub fn regenerate_api_key(
    conn: MainPGDatabase,
    request_data: Json<MeRequest>,
) -> Result<JsonValue, JsonValue> {
    let user_id = match account_services::is_authenticated(&request_data.token, &conn) {
        Ok(data) => data,
        Err(err) => {
            return Err(json!({
              "success": false,
              "message": err
            }))
        }
    };

    match account_services::regenerate_api_key(&user_id, conn) {
        Ok(data) => Ok(json!({
          "success": true,
          "api_key": data
        })),
        Err(err) => {
            println!("ERROR: {}", err);
            return Err(json!({
              "success": false,
              "api_key": "Something went wrong regenerating the api key"
            }));
        }
    }
}

#[post("/auth/register", format = "json", data = "<request_data>")]
pub fn register(conn: MainPGDatabase, request_data: Json<RegisterRequest>) -> JsonValue {
    if !verify_recaptcha(&request_data.captcha) {
        return json!({
          "success": false,
          "message": "Captcha validation failed, please try again."
        });
    };

    match account_services::register_user(
        &request_data.email,
        &request_data.username,
        &request_data.password,
        conn,
    ) {
        Ok(data) => {
            return json!({
              "success": true,
              "token": data
            });
        }
        Err(errmessage) => {
            return json!({
              "success": false,
              "message": errmessage
            });
        }
    }
}

fn verify_recaptcha(response: &String) -> bool {
    let data = ureq::post("https://www.google.com/recaptcha/api/siteverify").send_form(&[
        ("secret", &std::env::var("CAPTCHA_KEY").unwrap()),
        ("response", response),
    ]);

    let json = data.into_json().unwrap();

    let success: bool = json.get("success").unwrap().as_bool().unwrap();

    success
}

#[post("/auth/login", format = "json", data = "<request_data>")]
pub fn login(
    conn: MainPGDatabase,
    request_data: Json<LoginRequest>,
    remote_addr: SocketAddr,
    user_agent: UserAgent,
) -> JsonValue {
    if !verify_recaptcha(&request_data.captcha) {
        return json!({
          "success": false,
          "message": "Captcha validation failed, please try again."
        });
    };

    let UserAgent(useragent_string) = user_agent;
    match account_services::login(
        &request_data.username,
        &request_data.password,
        conn,
        remote_addr.to_string(),
        useragent_string,
    ) {
        Ok(data) => {
            return json!({
              "success": true,
              "token": data
            });
        }
        Err(errmessage) => {
            return json!({
              "success": false,
              "message": errmessage
            });
        }
    }
}
