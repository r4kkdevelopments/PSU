use rocket_contrib::json::{Json, JsonValue};

use crate::MainPGDatabase;
use serde::Deserialize;

use crate::modules::account_services;

#[derive(Deserialize)]
pub struct addPremiumReq {
    pub orderType: i8,
    pub token: String,
    pub target: String,
}

#[derive(Deserialize)]
pub struct removePremiumReq {
    pub token: String,
    pub target: String,
}

#[post("/auth/premium/get_premium", format = "json", data = "<request_data>")]
pub fn get_prem(
    conn: MainPGDatabase,
    request_data: Json<removePremiumReq>,
) -> Result<JsonValue, JsonValue> {
    // Check user is logged in
    let user_id = match account_services::is_authenticated(&request_data.token, &conn) {
        Ok(data) => data,
        Err(_err) => {
            return Err(json!({"success":false, "message": String::from("ERR_AUTH_FAILED")}))
        }
    };

    // Check user has permission to add premium
    let has_permission = match account_services::permissions::has_perms(
        &user_id,
        &"user.premium.get".to_string(),
        &conn,
        false,
    ) {
        Ok(data) => data,
        Err(_err) => {
            return Err(json!({"success":false, "message": String::from("ERR_INTERNAL_ERR")}))
        }
    };

    if !has_permission {
        return Err(json!({"success":false, "message": String::from("PERMISSION_DENIED")}));
    };

    match account_services::has_premium(&request_data.target, &conn) {
        Some(data) => return Ok(json!({"success":true, "data": data})),
        None => return Ok(json!({"success":true, "data": false})),
    }
}

#[post(
    "/auth/premium/remove_premium",
    format = "json",
    data = "<request_data>"
)]
pub fn remove_prem(
    conn: MainPGDatabase,
    request_data: Json<removePremiumReq>,
) -> Result<JsonValue, JsonValue> {
    // Check user is logged in
    let user_id = match account_services::is_authenticated(&request_data.token, &conn) {
        Ok(data) => data,
        Err(_err) => {
            return Err(json!({"success":false, "message": String::from("ERR_AUTH_FAILED")}))
        }
    };

    // Check user has permission to add premium
    let has_permission = match account_services::permissions::has_perms(
        &user_id,
        &"user.premium.set".to_string(),
        &conn,
        false,
    ) {
        Ok(data) => data,
        Err(_err) => {
            return Err(json!({"success":false, "message": String::from("ERR_INTERNAL_ERR")}))
        }
    };

    if !has_permission {
        return Err(json!({"success":false, "message": String::from("PERMISSION_DENIED")}));
    };

    match account_services::remove_premium(&request_data.target, &conn) {
        Ok(_data) => return Ok(json!({"success":true, "message": String::from("SUCCESS")})),
        Err(err) => return Err(json!({"success":false, "message": String::from(err)})),
    }
}

#[post("/auth/premium/add_premium", format = "json", data = "<request_data>")]
pub fn add_prem(
    conn: MainPGDatabase,
    request_data: Json<addPremiumReq>,
) -> Result<JsonValue, JsonValue> {
    // Check user is logged in
    let user_id = match account_services::is_authenticated(&request_data.token, &conn) {
        Ok(data) => data,
        Err(_err) => {
            return Err(json!({"success":false, "message": String::from("ERR_AUTH_FAILED")}))
        }
    };

    // Check user has permission to add premium
    let has_permission = match account_services::permissions::has_perms(
        &user_id,
        &"user.premium.set".to_string(),
        &conn,
        false,
    ) {
        Ok(data) => data,
        Err(_err) => {
            return Err(json!({"success":false, "message": String::from("ERR_INTERNAL_ERR")}))
        }
    };

    if !has_permission {
        return Err(json!({"success":false, "message": String::from("PERMISSION_DENIED")}));
    };

    match account_services::remove_premium(&request_data.target, &conn) {
        Ok(_data) => (),
        Err(err) => return Err(json!({"success":false, "message": String::from(err)})),
    }

    match account_services::add_premium(
        &request_data.target,
        &nanoid::nanoid!(),
        &"Dashboard".to_string(),
        &request_data.orderType,
        &conn,
    ) {
        Ok(_data) => return Ok(json!({"success":true, "message": String::from("SUCCESS")})),
        Err(err) => return Err(json!({"success":false, "message": String::from(err)})),
    }
}
