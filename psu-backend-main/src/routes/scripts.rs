use rocket::http::{ContentType, Status};
use rocket::response::status::Custom;
use rocket::Data;
use rocket_contrib::json::{Json, JsonValue};
use script_services::create_new_script;
use serde::Deserialize;

use crate::{modules::script_services, MainPGDatabase};

// #[post("/upload", data = "<data>")]
// signature requires the request to have a `Content-Type`
// pub fn multipart_upload(cont_type: &ContentType, data: Data) -> Result<String, Custom<String>> {
//     // this and the next check can be implemented as a request guard but it seems like just
//     // more boilerplate than necessary
//     if !cont_type.is_form_data() {
//         return Err(Custom(
//             Status::BadRequest,
//             "Content-Type not multipart/form-data".into(),
//         ));
//     }
//
//     let (_, boundary) = cont_type
//         .params()
//         .find(|&(k, _)| k == "boundary")
//         .ok_or_else(|| {
//             Custom(
//                 Status::BadRequest,
//                 "`Content-Type: multipart/form-data` boundary param not provided".into(),
//             )
//         })?;
//
//     match script_services::process_upload_aws(boundary, data) {
//         Ok(resp) => Ok(resp),
//         Err(err) => Err(Custom(Status::InternalServerError, err.to_string())),
//     }
// }

// having a streaming output would be nice; there's one for returning a `Read` impl
// but not one that you can `write()` to

#[post("/scripts/updateScript", data = "<data>")]
pub fn update_script(
    cont_type: &ContentType,
    data: Data,
    conn: MainPGDatabase,
) -> Result<JsonValue, Custom<JsonValue>> {
    if !cont_type.is_form_data() {
        return Err(Custom(
            Status::BadRequest,
            json!({
              "success": false,
              "message": "Content-Type not multipart/form-data"
            }),
        ));
    }

    let (_, boundary) = cont_type
        .params()
        .find(|&(k, _)| k == "boundary")
        .ok_or_else(|| {
            Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": "Content-Type: multipart/form-data` boundary param not provided"
                }),
            )
        })?;

    let script_id = match script_services::update_script(&boundary, &conn, data) {
        Ok(result) => result,
        Err(err) => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": err
                }),
            ))
        }
    };

    Ok(json!({
      "success": true,
      "scriptID": script_id
    }))
}

#[post("/scripts/createScript", data = "<data>")]
pub fn create_script(
    cont_type: &ContentType,
    data: Data,
    conn: MainPGDatabase,
) -> Result<JsonValue, Custom<JsonValue>> {
    if !cont_type.is_form_data() {
        return Err(Custom(
            Status::BadRequest,
            json!({
              "success": false,
              "message": "Content-Type not multipart/form-data"
            }),
        ));
    }

    let (_, boundary) = cont_type
        .params()
        .find(|&(k, _)| k == "boundary")
        .ok_or_else(|| {
            Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": "Content-Type: multipart/form-data` boundary param not provided"
                }),
            )
        })?;

    // Convert our data into a field hashmap.
    let form_fields = match script_services::process_multipart(boundary, data) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": "Failed to process multidata. Is it corrupted?"
                }),
            ));
        }
    };

    // Perform checks on fields
    if !(form_fields.contains_key("title")
        && form_fields.contains_key("description")
        && form_fields.contains_key("public")
        && form_fields.contains_key("token"))
    {
        return Err(Custom(
            Status::BadRequest,
            json!({
              "success": false,
              "message": "Didn't recieve all required fields. Rejecting"
            }),
        ));
    }

    let script = match script_services::Script::fields_to_self(&form_fields) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": "Something went wrong processing the form"
                }),
            ));
        }
    };

    let script_id = match create_new_script(&conn, script, boundary) {
        Ok(result) => result,
        Err(err) => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": err
                }),
            ))
        }
    };

    Ok(json!({
      "success": true,
      "scriptID": script_id
    }))
}

#[derive(Deserialize)]
pub struct UpdateScriptRequest {
    pub token: String,
}

// #[post("/scripts/updateScript", format = "json", data = "<request_data>")]
// pub fn update_script(conn: MainPGDatabase, request_data: Json<UpdateScriptRequest>) -> Result<JsonValue, Custom<JsonValue>> {
//   Ok(json!({
//     "success": true,
//     "message": "Successfully updated script"
//   }))
// }

#[post(
    "/scripts/private/getAllScripts",
    format = "json",
    data = "<request_data>"
)]
pub fn get_all_scripts(
    conn: MainPGDatabase,
    request_data: Json<UpdateScriptRequest>,
) -> Result<JsonValue, Custom<JsonValue>> {
    let data = match script_services::get_private_scripts(&request_data.token, &conn) {
        Ok(data) => data,
        Err(err) => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": err.to_string()
                }),
            ));
        }
    };

    Ok(json!({
      "success": true,
      "data": data
    }))
}

#[derive(Deserialize)]
pub struct updatePubScriptRequest {
    pub token: String,
    pub script_id: String,
    pub new_location: String,
}

#[post(
    "/scripts/updatePublicScript",
    format = "json",
    data = "<request_data>"
)]
pub fn update_pub_script(
    conn: MainPGDatabase,
    request_data: Json<updatePubScriptRequest>,
) -> Result<JsonValue, Custom<JsonValue>> {
    return match script_services::update_public_script(
        &conn,
        &request_data.token,
        &request_data.script_id,
    ) {
        Ok(_) => Ok(json!({"success": true, "message": "SUCCESS"})),
        Err(err) => Err(Custom(
            Status::BadRequest,
            json!({"success": false, "message": err}),
        )),
    };
}

#[post(
    "/scripts/public/getAllScripts",
    format = "json",
    data = "<request_data>"
)]
pub fn get_all_scripts_pub(
    conn: MainPGDatabase,
    request_data: Json<UpdateScriptRequest>,
) -> Result<JsonValue, Custom<JsonValue>> {
    let data = match script_services::get_public_scripts(&conn, &request_data.token) {
        Ok(data) => data,
        Err(err) => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": err.to_string()
                }),
            ));
        }
    };

    Ok(json!({
      "success": true,
      "data": data
    }))
}

#[derive(Deserialize)]
pub struct GetScriptRequest {
    pub token: String,
    pub scriptID: String,
}

#[post("/scripts/getScript", format = "json", data = "<request_data>")]
pub fn get_script(
    conn: MainPGDatabase,
    request_data: Json<GetScriptRequest>,
) -> Result<Vec<u8>, Custom<JsonValue>> {
    let data = match script_services::get_script(&request_data.token, &request_data.scriptID, &conn)
    {
        Ok(data) => data,
        Err(err) => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": err.to_string()
                }),
            ));
        }
    };

    Ok(data)
}

#[derive(Deserialize)]
pub struct DeleteScriptRequest {
    pub token: String,
    pub scriptID: String,
}

#[post("/scripts/deleteScript", format = "json", data = "<request_data>")]
pub fn delete_script(
    conn: MainPGDatabase,
    request_data: Json<DeleteScriptRequest>,
) -> Result<JsonValue, Custom<JsonValue>> {
    match script_services::delete_script(&conn, &request_data.scriptID, &request_data.token) {
        Ok(data) => data,
        Err(err) => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": err.to_string()
                }),
            ));
        }
    };

    Ok(json!({
      "success": true,
      "message": "SUCCESS"
    }))
}
