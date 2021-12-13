use multipart::server::{
    save::{SaveResult::*, SavedData, SavedField},
    Multipart,
};
use postgres::rows::Rows;
use rocket::data::Data;
use serde::Serialize;
use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

use std::io::prelude::*;

use rusoto_core::{credential::ChainProvider, request::HttpClient, Region};
use rusoto_s3::{DeleteObjectRequest, GetObjectRequest, PutObjectRequest, S3Client, S3};

use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::modules::account_services;
use crate::MainPGDatabase;

use nanoid::nanoid;

pub fn field_to_string(lmao: &SavedField) -> Result<String, Error> {
    let data: String = match &lmao.data {
        SavedData::Text(data) => data.to_owned(),
        SavedData::File(_data, _int) => return Err(Error::new(ErrorKind::Other, "Invalid Type")),
        SavedData::Bytes(_data) => return Err(Error::new(ErrorKind::Other, "Invalid Type")),
    };

    Ok(data)
}

pub fn field_to_file(field: &SavedField) -> Result<Vec<u8>, Error> {
    let mut buffer: Vec<u8> = Default::default();
    let mut data = field.data.readable()?;

    data.read_to_end(&mut buffer)?;

    return Ok(buffer);
}

pub fn field_to_bool(lmao: &SavedField) -> Result<bool, Error> {
    let data: bool = match &lmao.data {
        SavedData::Text(data) => match data.to_lowercase().as_str() {
            "true" => true,
            "false" => false,
            _ => false,
        },
        SavedData::File(_data, _int) => return Err(Error::new(ErrorKind::Other, "Invalid Type")),
        SavedData::Bytes(_data) => return Err(Error::new(ErrorKind::Other, "Invalid Type")),
    };

    Ok(data)
}

#[derive(Debug)]
pub struct Script {
    pub title: String,
    pub description: String,
    pub public: bool,
    pub token: String,
    pub file: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct SafeScript {
    pub title: String,
    pub description: String,
    pub public: bool,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct PubSafeScript {
    pub title: String,
    pub description: String,
    pub public: bool,
    pub id: String,
    pub location: String,
}

impl Script {
    pub fn fields_to_self(fields: &HashMap<Arc<str>, Vec<SavedField>>) -> Result<Self, Error> {
        Ok(Self {
            title: match fields.get("title") {
                Some(data) => field_to_string(&data[0])?,
                None => {
                    return Err(Error::new(ErrorKind::Other, "File Field not supplied!"));
                }
            },
            description: match fields.get("description") {
                Some(data) => field_to_string(&data[0])?,
                None => {
                    return Err(Error::new(ErrorKind::Other, "File Field not supplied!"));
                }
            },
            public: match fields.get("public") {
                Some(data) => field_to_bool(&data[0])?,
                None => {
                    return Err(Error::new(ErrorKind::Other, "File Field not supplied!"));
                }
            },
            token: match fields.get("token") {
                Some(data) => field_to_string(&data[0])?,
                None => {
                    return Err(Error::new(ErrorKind::Other, "File Field not supplied!"));
                }
            },
            file: match fields.get("file") {
                Some(data) => field_to_file(&data[0])?,
                None => {
                    return Err(Error::new(ErrorKind::Other, "File Field not supplied!"));
                }
            },
        })
    }
}

pub fn delete_script(
    conn: &MainPGDatabase,
    script_id: &str,
    token: &String,
) -> Result<String, String> {
    let user_id = match account_services::is_authenticated(token, conn) {
        Ok(data) => data,
        Err(_err) => return Err(String::from("ERR_AUTH_FAILED")),
    };

    // Get the script and check the user can delete it.
    let rows_recieved: Rows = match conn.query(
        r#"SELECT * FROM lunar_buffxnte_psu.scripts WHERE id = $1 LIMIT 1"#,
        &[&script_id],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    if rows_recieved.len() < 1 {
        return Err(String::from("No with that ID found"));
    };

    let script_owner_id: String = rows_recieved.get(0).get("belongs_to");

    if script_owner_id != user_id {
        return Err(String::from("ERR_INVALID_AUTH"));
    };

    // Checks finished. Start deleting.
    match conn.execute(
        "DELETE FROM lunar_buffxnte_psu.scripts WHERE id = $1;",
        &[&script_id],
    ) {
        Ok(_data) => (),
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    match delete_object_aws(script_id.to_owned()) {
        Ok(_data) => (),
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_AWS_ERR"));
        }
    };

    Ok(String::from("SUCCESS"))
}

pub fn update_public_script(
    conn: &MainPGDatabase,
    token: &str,
    script_id: &str,
) -> Result<String, String> {
    // Check Auth
    let user_id = match account_services::is_authenticated(&token.to_owned(), conn) {
        Ok(data) => data,
        Err(_err) => return Err(String::from("ERR_AUTH_FAILED")),
    };

    // Check if UUID is valid
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"^[0-9A-F]{8}-[0-9A-F]{4}-4[0-9A-F]{3}-[89AB][0-9A-F]{3}-[0-9A-F]{12}$"#)
                .unwrap();
    }


    //TODO: Check AWS to see if object exists.

    // Get script
    let rows_recieved: Rows = match conn.query(
        r#"SELECT * FROM lunar_buffxnte_psu.scripts WHERE id = $1 LIMIT 1"#,
        &[&script_id],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    if rows_recieved.len() < 1 {
        return Err(String::from("Script doesn't exist"));
    }

    let script_owner: String = rows_recieved.get(0).get("belongs_to");

    if user_id != script_owner {
        return Err(String::from("ERR_AUTH_FAILED"));
    };

    // Check if script is public
    let is_public: bool = rows_recieved.get(0).get("public");

    if !is_public {
        return Err(String::from("ERR_SCRIPT_NOT_PUBLIC"));
    }

    // Delete all instances of previous public script with this ID
    match conn.execute(
        "DELETE FROM lunar_buffxnte_psu.public_scripts WHERE id = $1;",
        &[&script_id],
    ) {
        Ok(_) => (),
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    // Create a new entry in public scripts
    match conn.execute(
        "INSERT INTO lunar_buffxnte_psu.public_scripts(id, location) VALUES ($1, $2);",
        &[&script_id],
    ) {
        Ok(_) => (return Ok(String::from("SUCCESS"))),
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };
}

pub fn get_public_scripts(
    conn: &MainPGDatabase,
    token: &String,
) -> Result<Vec<PubSafeScript>, String> {
    // Check auth
    match account_services::is_authenticated(&token, conn) {
        Ok(_) => (),
        Err(_err) => return Err(String::from("ERR_AUTH_FAILED")),
    };

    let raw_public_scripts: Rows = match conn.query(
        "SELECT id, location FROM lunar_buffxnte_psu.public_scripts;",
        &[],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("SQL ERROR: {}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    let public_script_metadata: Rows = match conn.query("SELECT * FROM lunar_buffxnte_psu.scripts WHERE id = ANY(SELECT id from lunar_buffxnte_psu.public_scripts);", &[]) {
      Ok(data) => data,
      Err(err) => {
        println!("SQL ERROR: {}", err);
        return Err(String::from("ERR_INTERNAL_ERR"));
      }
    };

    let mut scripts: Vec<PubSafeScript> = Default::default();

    for i in 0..raw_public_scripts.len() {
        // Find ID in vec
        let mut location = String::from("");
        for x in &raw_public_scripts {
            let id: String = x.get("id");
            let compare_id: String = public_script_metadata.get(i).get("id");

            if id == compare_id {
                location = x.get("location")
            }
        }

        let script = PubSafeScript {
            title: public_script_metadata.get(i).get("title"),
            description: public_script_metadata.get(i).get("description"),
            public: public_script_metadata.get(i).get("public"),
            id: public_script_metadata.get(i).get("id"),
            location: location,
        };
        scripts.push(script)
    }

    return Ok(scripts);
}

pub fn create_new_script(
    conn: &MainPGDatabase,
    script: Script,
    _boundary: &str,
) -> Result<String, String> {
    // Check auth
    let user_id = match account_services::is_authenticated(&script.token, conn) {
        Ok(data) => data,
        Err(_err) => return Err(String::from("ERR_AUTH_FAILED")),
    };

    // See how many scripts the user already has
    let current_scripts = match get_private_scripts(&script.token, conn) {
        Ok(data) => data,
        Err(_err) => return Err(String::from("ERR_AUTH_FAILED")),
    };

    if current_scripts.len() >= 50 {
        return Err(String::from("ERR_MAX_SCRIPTS_EXCEEDED"));
    }

    // Upload Script to AWS and get ID
    let script_id = match process_upload_aws(script.file, None) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    // Now create a entry in our database
    match conn.execute(
        r#"INSERT INTO lunar_buffxnte_psu.scripts(
      title, description, updated_at, created_at, public, "belongs_to", id)
      VALUES ($1, $2, $3, $4, $5, $6, $7);"#,
        &[
            &script.title,
            &script.description,
            &chrono::Utc::now(),
            &chrono::Utc::now(),
            &script.public,
            &user_id,
            &script_id,
        ],
    ) {
        Ok(_data) => return Ok(String::from(&script_id)),
        Err(err) => {
            println!("{}", err);
            return Err(String::from("Something went wrong creating the script"));
        }
    };
}

pub fn process_multipart(
    boundary: &str,
    data: Data,
) -> Result<HashMap<Arc<str>, Vec<SavedField>>, String> {
    match Multipart::with_body(data.open(), boundary)
        .save()
        .force_text()
        .with_dir("./temp/")
    {
        Full(entries) => return Ok(entries.fields),
        Partial(partial, reason) => {
            println!("Request partially processed: {:?}", reason);
            if let Some(field) = partial.partial {
                println!("Stopped on field: {:?}", field.source.headers);
            }

            Err(String::from(
                "Error: Got partial multipart. Wanted Full. Possible multipart corruption",
            ))
        }
        Error(_e) => return Err(String::from("Something went wrong parsing multipart")),
    }
}

pub fn delete_object_aws(script_id: String) -> Result<String, String> {
    let mut chain = ChainProvider::new();
    chain.set_timeout(Duration::from_millis(200));

    let s3cli = S3Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        chain,
        Region::Custom {
            name: "nyc-3".to_owned(),
            endpoint: "https://psu.sfo3.digitaloceanspaces.com".to_owned(),
        },
    );

    let rt = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            println!("Tokio Runtime Error: {}", err);
            return Err(String::from(
                "Tokio Runtime Error. Please notify the administrator.",
            ));
        }
    };

    let result = rt.block_on(async {
        let test = s3cli
            .delete_object(DeleteObjectRequest {
                bucket: String::from("psu-scripts-bucket"),
                key: format!("{}", script_id),
                ..Default::default()
            })
            .await;

        test
    });

    match result {
        Ok(_data) => Ok(script_id),
        Err(err) => {
            println!("AWS ERROR: {}", err);
            Err(String::from(
                "A AWS Error occoured. Please notify the administrator.",
            ))
        }
    }
}

pub fn process_upload_aws(data: Vec<u8>, script_id: Option<String>) -> Result<String, String> {
    println!("testing");
    let mut chain = ChainProvider::new();
    chain.set_timeout(Duration::from_millis(200));

    let s3cli = S3Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        chain,
        Region::Custom {
            name: "nyc-3".to_owned(),
            endpoint: "https://psu.sfo3.digitaloceanspaces.com".to_owned(),
        },
    );

    let id;

    match script_id {
        Some(script_id) => id = script_id,
        None => {
            id = nanoid!();
        }
    }

    // BAD DUMB ASS CODE. RESULT OF NEEDING TO EXECUTE ASYNC CODE IN NON-ASYNC FUNCTIONS. DUE TO ROCKET NOT SUPPORTING ASYNC ROUTES.
    let rt = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            println!("Tokio Runtime Error: {}", err);
            return Err(String::from(
                "Tokio Runtime Error. Please notify the administrator.",
            ));
        }
    };

    let result = rt.block_on(async {
        let test = s3cli
            .put_object(PutObjectRequest {
                bucket: String::from("psu-scripts-bucket"),
                key: format!("{}", id),
                body: Some(data.into()),
                ..Default::default()
            })
            .await;

        test
    });

    match result {
        Ok(_data) => Ok(id),
        Err(err) => {
            println!("AWS ERROR: {}", err);
            Err(String::from(
                "A AWS Error occoured. Please notify the administrator.",
            ))
        }
    }
}

pub fn get_private_scripts(
    token: &String,
    conn: &MainPGDatabase,
) -> Result<Vec<SafeScript>, String> {
    let user_id = match account_services::is_authenticated(token, conn) {
        Ok(data) => data,
        Err(_err) => return Err(String::from("ERR_AUTH_FAILED")),
    };

    let rows_recieved: Rows = match conn.query(
        r#"SELECT * FROM lunar_buffxnte_psu.scripts WHERE "belongs_to" = $1"#,
        &[&user_id],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    let mut scripts: Vec<SafeScript> = Default::default();

    for i in 0..rows_recieved.len() {
        let script = SafeScript {
            title: rows_recieved.get(i).get("title"),
            description: rows_recieved.get(i).get("description"),
            public: rows_recieved.get(i).get("public"),
            id: rows_recieved.get(i).get("id"),
        };
        scripts.push(script)
    }

    Ok(scripts)
}

pub fn update_script(
    boundary: &str,
    conn: &MainPGDatabase,
    script: Data,
) -> Result<String, String> {
    let multipart_data = match process_multipart(boundary, script) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err(String::from("Something went wrong parsing multipart data."));
        }
    };

    let file_field = match multipart_data.get("file") {
        Some(data) => data,
        None => return Err(String::from("No file field was recieved")),
    };

    let token_field = match multipart_data.get("token") {
        Some(data) => data,
        None => return Err(String::from("No token field was recieved")),
    };

    let token = match field_to_string(&token_field[0]) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err(String::from("Something went wrong processing the file."));
        }
    };

    let script_id_field = match multipart_data.get("scriptID") {
        Some(data) => data,
        None => return Err(String::from("No scriptID field was recieved")),
    };

    let script_id = match field_to_string(&script_id_field[0]) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err(String::from("Something went wrong processing the file."));
        }
    };

    // Check user's auth.
    let user_id = match account_services::is_authenticated(&token, conn) {
        Ok(data) => data,
        Err(_err) => return Err(String::from("ERR_AUTH_FAILED")),
    };

    let rows_recieved: Rows = match conn.query(
        r#"SELECT * FROM lunar_buffxnte_psu.scripts WHERE id = $1 LIMIT 1"#,
        &[&script_id],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    if rows_recieved.len() < 1 {
        return Err(String::from("Script doesn't exist"));
    }

    let script_owner: String = rows_recieved.get(0).get("belongs_to");

    if user_id != script_owner {
        return Err(String::from("ERR_AUTH_FAILED"));
    };

    let file = match field_to_file(&file_field[0]) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err(String::from("Something went wrong processing the file."));
        }
    };

    match process_upload_aws(file, Some(script_id)) {
        Ok(_data) => return Ok(String::from("SUCCESS")),
        Err(err) => {
            println!("AWS ERROR: {}", err);
            return Err(String::from("AWS ERROR! Please contact the administrator."));
        }
    };
}

pub fn get_script(
    token: &String,
    script_id: &String,
    conn: &MainPGDatabase,
) -> Result<Vec<u8>, String> {
    let user_id = match account_services::is_authenticated(token, conn) {
        Ok(data) => data,
        Err(_err) => return Err(String::from("ERR_AUTH_FAILED")),
    };

    let rows_recieved: Rows = match conn.query(
        r#"SELECT * FROM lunar_buffxnte_psu.scripts WHERE id = $1 LIMIT 1"#,
        &[&script_id],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    if rows_recieved.len() < 1 {
        return Err(String::from("Script doesn't exist"));
    }

    let script_owner: String = rows_recieved.get(0).get("belongs_to");

    if user_id != script_owner {
        return Err(String::from("ERR_AUTH_FAILED"));
    };

    let mut chain = ChainProvider::new();
    chain.set_timeout(Duration::from_millis(200));

    let s3cli = S3Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        chain,
        Region::Custom {
            name: "nyc-3".to_owned(),
            endpoint: "https://psu.sfo3.digitaloceanspaces.com".to_owned(),
        },
    );

    // Now get the file
    let rt = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            println!("Tokio Runtime Error: {}", err);
            return Err(String::from(
                "Tokio Runtime Error. Please notify the administrator.",
            ));
        }
    };

    let result = rt.block_on(async {
        let request = s3cli
            .get_object(GetObjectRequest {
                key: script_id.to_owned(),
                bucket: String::from("psu-scripts-bucket"),
                ..Default::default()
            })
            .await;

        request
    });

    let data = match result {
        Ok(data) => data,
        Err(err) => {
            println!("AWS ERROR: {}", err);
            return Err(String::from(
                "A AWS Error occoured. Please notify the administrator.",
            ));
        }
    };

    let byte_stream = match data.body {
        Some(data) => data,
        None => return Err(String::from("Script doesn't exist")),
    };

    let mut buffer = Vec::new();
    match byte_stream.into_blocking_read().read_to_end(&mut buffer) {
        Ok(_data) => {}
        Err(err) => {
            println!("ERROR: {}", err);
            return Err(String::from(
                "Something went wrong recieving the file from AWS",
            ));
        }
    };

    Ok(buffer.iter().cloned().collect())
}
