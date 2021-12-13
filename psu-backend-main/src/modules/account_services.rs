use crate::modules::user;
use crate::MainPGDatabase;

use bcrypt::{hash, verify};
use chrono::Duration;
use nanoid::nanoid;
use postgres::rows::Rows;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use user::row_to_user;

// MultiPart bullshit
use multipart::server::{
    save::{DataReader, Entries, SaveResult::*, SavedData, SavedField, TempDir},
    Multipart,
};
use std::collections::HashMap;
use std::sync::Arc;

use rusoto_core::{credential::ChainProvider, request::HttpClient, Region};
use rusoto_s3::{DeleteObjectRequest, GetObjectRequest, PutObjectRequest, S3Client, S3};
use tokio::runtime::Runtime;

pub mod permissions;
pub mod roles;

pub fn has_premium(user_id: &String, conn: &MainPGDatabase) -> Option<String> {
    // Make sure transaction ID hasn't already been used
    let rows_recieved: Rows = match conn.query("SELECT * FROM lunar_buffxnte_psu.purchases WHERE expires_at > now()::date AND user_id = $1;", &[&user_id]) {
    Ok(data) => data,
    Err(err) => {
        println!("Error: {:?}", err);
        return None;
    }
  };

    if rows_recieved.len() == 0 {
        return None;
    } else {
        let expires_at: chrono::DateTime<chrono::Utc> = rows_recieved.get(0).get("expires_at");
        return Some(expires_at.to_string());
    }
}

pub fn finalise_password_reset(
    token: &String,
    password: &String,
    conn: &MainPGDatabase,
) -> Result<String, String> {
    let rows_recieved: Rows = match conn.query(
    r#"SELECT email, token, created_at, user_id FROM lunar_buffxnte_psu.password_resets WHERE created_at BETWEEN NOW() - INTERVAL '30 MINUTES' AND NOW() AND token = $1;"#,
    &[&token],
  ) {
      Ok(data) => data,
      Err(err) => {
          println!("{:?}", err);
          return Err(String::from("ERR_INTERNAL_ERR"));
      }
  };

    if rows_recieved.len() <= 0 {
        return Err(String::from("Invalid Token."));
    };

    // Everything is valid so get user ID and rewrite the password
    let user_id: String = rows_recieved.get(0).get("user_id");

    match conn.execute(
        "UPDATE lunar_buffxnte_psu.users SET password=$1 WHERE id = $2;",
        &[&hash(password, 12).unwrap(), &user_id],
    ) {
        Ok(_data) => (),
        Err(err) => {
            println!("SQL ERROR: {}", err);
            return Err(String::from("Something went wrong resetting the passwsord"));
        }
    };

    return Ok(String::from("Successfully Reset Password"));
}

pub fn process_multipart(
    boundary: &str,
    data: rocket::Data,
) -> Result<HashMap<Arc<str>, Vec<SavedField>>, String> {
    match Multipart::with_body(data.open(), boundary)
        .save()
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

pub fn process_upload_aws(data: Vec<u8>, fileID: &String) -> Result<String, String> {
    let mut chain = ChainProvider::new();
    chain.set_timeout(std::time::Duration::from_millis(200));

    let s3cli = S3Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        chain,
        Region::Custom {
            name: "nyc-3".to_owned(),
            endpoint: "https://psu.sfo3.digitaloceanspaces.com".to_owned(),
        },
    );

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
                bucket: String::from("psu-public"),
                key: format!("profile_pictures/{}.png", fileID),
                body: Some(data.into()),
                ..Default::default()
            })
            .await;

        test
    });

    match result {
        Ok(_data) => Ok(fileID.to_owned()),
        Err(err) => {
            println!("AWS ERROR: {}", err);
            Err(String::from(
                "A AWS Error occoured. Please notify the administrator.",
            ))
        }
    }
}

pub fn process_avatar_upload(
    boundary: &str,
    data: rocket::Data,
    conn: &MainPGDatabase,
) -> Result<String, String> {
    let multipart_data = match process_multipart(boundary, data) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err(String::from("Something went wrong parsing multipart data."));
        }
    };

    let token_field = match multipart_data.get("token") {
        Some(data) => data,
        None => return Err(String::from("No token field was recieved")),
    };

    let token = script_services::field_to_string(&token_field[0]).unwrap();

    // Check user's auth.
    let user_id = match is_authenticated(&token, conn) {
        Ok(data) => data,
        Err(_err) => return Err(String::from("ERR_AUTH_FAILED")),
    };

    let file_field = match multipart_data.get("file") {
        Some(data) => data,
        None => return Err(String::from("No file field was recieved")),
    };

    let file = match script_services::field_to_file(&file_field[0]) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err(String::from("Something went wrong processing the file."));
        }
    };

    let avatar_id = nanoid!(40);

    match process_upload_aws(file, &avatar_id) {
        Ok(_data) => (),
        Err(err) => {
            println!("AWS ERROR: {}", err);
            return Err(String::from("AWS ERROR! Please contact the administrator."));
        }
    };

    match conn.execute(
        "UPDATE lunar_buffxnte_psu.users SET avatar=$1 WHERE id = $2;",
        &[
            &format!("https://cdn.psu.dev/profile_pictures/{}.png", &avatar_id),
            &user_id,
        ],
    ) {
        Ok(_data) => return Ok("OK".to_string()),
        Err(err) => {
            println!("SQL ERROR! {}", err);
            return Err(
                "Something went wrong while uploading this image. Please try again later."
                    .to_string(),
            );
        }
    };
}

pub fn update_profile(
    user_id: &String,
    email: &String,
    first_name: &String,
    last_name: &String,
    conn: &MainPGDatabase,
) -> Result<String, String> {
    // Run veriifcation
    if !check_email(email) {
        return Err("Invalid Email Address!".to_string());
    }

    if first_name.len() < 3 || last_name.len() < 3 {
        return Err("Invalid Last or First Name!".to_string());
    }

    match conn.execute(
        "UPDATE lunar_buffxnte_psu.users SET email=$1,first_name=$2, last_name=$3 WHERE id = $4;",
        &[&email, &first_name, &last_name, &user_id],
    ) {
        Ok(_data) => return Ok("Successfully updated profile!".to_string()),
        Err(err) => {
            println!("SQL ERROR! {}", err);
            return Err(
                "Something went wrote updating this profile. Please contact CTO Err: SQL_ERR"
                    .to_string(),
            );
        }
    }
}

pub fn send_reset_email(email_address: &String, conn: &MainPGDatabase) -> Result<String, String> {
    // Search for user...
    let rows_recieved: Rows = match conn.query(
        r#"SELECT * FROM lunar_buffxnte_psu.users WHERE email = $1 ORDER BY id ASC LIMIT 1"#,
        &[&email_address],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    if rows_recieved.len() <= 0 {
        return Ok(String::from("User doesn't exist")); // We return true because we don't need to validate to the user if the request succeeded. For security reasons
    };

    let user = row_to_user(&rows_recieved.get(0));

    // if valid user then generate a NANOID
    let token = nanoid::nanoid!();

    match conn.execute(
        "INSERT INTO lunar_buffxnte_psu.password_resets(
      email, token, created_at, user_id)
      VALUES ($1, $2, $3, $4);",
        &[&email_address, &token, &chrono::Utc::now(), &user.id],
    ) {
        Ok(_data) => (),
        Err(err) => {
            println!("SQL ERROR: {}", err);
            return Err(String::from(
                "Something went wrong creating the password reset token",
            ));
        }
    };

    // Send the email
    let data = ureq::post("https://api.mailgun.net/v3/email.psu.dev/messages")
        .set(
            "Authorization",
            &format!(
                "Basic {}",
                base64::encode(format!("api:{}", std::env::var("MAILGUN_KEY").unwrap()))
            ),
        )
        .send_form(&[
            ("from", "PSU <donotreply@email.psu.dev>"),
            ("to", &format!("test user <{}>", email_address)),
            ("subject", "PSU Password Reset"),
            ("template", "password-reset-template"),
            ("v:username", &user.username.unwrap()),
            ("v:password_reset_token", &token),
        ]);

    Ok("SUCCESS".to_string())
}

pub fn add_premium(
    user_id: &String,
    order_id: &String,
    method: &String,
    order_type: &i8,
    conn: &MainPGDatabase,
) -> Result<String, String> {
    // Make sure transaction ID hasn't already been used
    let rows_recieved: Rows = match conn.query(
        "SELECT id FROM lunar_buffxnte_psu.purchases WHERE txn_id = $1;",
        &[&order_id],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("Error: {:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    if rows_recieved.len() > 0 {
        return Err(String::from("ERR_ID_ALREADY_USED"));
    };

    // Now add premium
    let order_amount = match order_type {
        0 => 6.49,
        1 => 29.99,
        2 => 74.99,
        _ => 0.00,
    };

    let date_expires: chrono::DateTime<chrono::Utc> = match order_type {
        0 => chrono::Utc::now() + Duration::days(30),
        1 => chrono::Utc::now() + Duration::days(365),
        2 => chrono::Utc::now() + Duration::days(99999),
        _ => chrono::Utc::now() + Duration::days(1),
    };

    match conn.execute(
        "INSERT INTO lunar_buffxnte_psu.purchases(
      txn_id, method, user_id, status, amount, active, chargebacked, created_at, expires_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);",
        &[
            &order_id,
            &method,
            &user_id,
            &(1 as i16),
            &order_amount,
            &(1 as i16),
            &(0 as i16),
            &chrono::Utc::now(),
            &date_expires,
        ],
    ) {
        Ok(_data) => return Ok(String::from("SUCCESSFULLY ACTIVATED PREMIUM")),
        Err(err) => {
            println!("SQL ERROR: {}", err);
            return Err(String::from("Something went wrong activating premium"));
        }
    };
}

pub fn remove_premium(user_id: &String, conn: &MainPGDatabase) -> Result<String, String> {
    match conn.execute(
        "DELETE FROM lunar_buffxnte_psu.purchases WHERE user_id = $1;",
        &[&user_id],
    ) {
        Ok(_data) => return Ok(String::from("SUCCESSFULLY REMOVED PREMIUM")),
        Err(err) => {
            println!("SQL ERROR: {}", err);
            return Err(String::from("Something went wrong removing premium"));
        }
    };
}

pub fn is_authenticated(token: &String, conn: &MainPGDatabase) -> Result<String, String> {
    // Get current token ID
    let rows_recieved: Rows = match conn.query("SELECT * from lunar_buffxnte_psu.sessions WHERE id = $1", &[&token]) {
        Ok(data) => data,
        Err(err) => {
            println!("Error: {:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    if rows_recieved.len() <= 0 {
        return Err(String::from("ERR_INVALID_TOKEN"));
    }

    let single_row_recieved = rows_recieved.get(0);

    let user_id: Option<String> = single_row_recieved.get("user_id");

    match user_id {
        Some(id) => return Ok(id),
        None => return Err(String::from("ERR_INVALID_TOKEN")),
    }
}

pub fn get_user(token: &String, conn: &MainPGDatabase) -> Result<user::User, String> {
    let user_id = match is_authenticated(token, &conn) {
        Ok(data) => data,
        Err(err) => return Err(err),
    };

    let rows_recieved: Rows = match conn.query(
        r#"SELECT * FROM lunar_buffxnte_psu.users WHERE id = $1 ORDER BY id ASC LIMIT 1"#,
        &[&user_id],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    if rows_recieved.len() <= 0 {
        return Err(String::from("ERR_INVALID_CRED"));
    }

    let user_row = rows_recieved.get(0);

    Ok(user::row_to_user(&user_row))
}

pub fn regenerate_api_key(user_id: &String, conn: MainPGDatabase) -> Result<String, String> {
    let new_key = nanoid!(50);

    // Remove previous ones.

    match conn.execute(
        "DELETE FROM lunar_buffxnte_psu.api_keys WHERE uid = $1;",
        &[&user_id],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("SQL ERROR: {}", err);
            return Err(String::from("Something went wrong creating the token"));
        }
    };

    match conn.execute(
    "INSERT INTO lunar_buffxnte_psu.api_keys(
    api_key, uid, todays_requests, allowed_requests, total_requests, last_request, created_at, disabled)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8);", &[
        &new_key,
        &user_id,
        &(0 as i64),
        &(100 as i64),
        &(0 as i64),
        &chrono::Utc::now().to_string(),
        &chrono::Utc::now().to_string(),
        &(0 as i16)
      ]) {
        Ok(data) => data,
        Err(err) => {
          println!("SQL ERROR: {}", err);
          return Err(String::from("Something went wrong creating the token"));
        }
      };

    Ok(new_key)
}

pub fn get_api_key(token: &String, conn: MainPGDatabase) -> Result<String, String> {
    let user_id = match is_authenticated(token, &conn) {
        Ok(string) => string,
        Err(_err) => {
            return Err(String::from("ERR_INVALID_AUTH"));
        }
    };

    let rows_recieved: Rows = match conn.query(
        r#"SELECT * FROM lunar_buffxnte_psu.api_keys WHERE uid = $1 LIMIT 1"#,
        &[&user_id],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    let api_key: String;

    if rows_recieved.len() <= 0 {
        api_key = match regenerate_api_key(&user_id, conn) {
            Ok(data) => data,
            Err(err) => {
                println!("ERROR: {}", err);
                return Err(String::from(
                    "Something went wrong regenerating the api key",
                ));
            }
        };
    } else {
        api_key = rows_recieved.get(0).get("api_key")
    }

    return Ok(api_key);
}

use regex::Regex;

use super::script_services;

fn check_email(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap();
    }
    RE.is_match(text)
}

pub fn register_user(
    email: &String,
    username: &String,
    password: &String,
    conn: MainPGDatabase,
) -> Result<String, String> {
    // Perform Validations.
    if !check_email(&email) {
        return Err("Email address is not valid.".to_string());
    };

    // Check if email address already exists
    let users_with_email: Rows = conn
        .query(
            "SELECT email FROM lunar_buffxnte_psu.users WHERE email = $1;",
            &[&email],
        )
        .unwrap();

    if users_with_email.len() != 0 {
        return Err("Email address already exists.".to_string());
    }

    // Perform password validations
    let score_estimate = zxcvbn::zxcvbn(&password, &[]).unwrap();

    if score_estimate.score() <= 2 {
        match score_estimate.feedback() {
            Some(data) => match data.warning() {
                Some(warning) => return Err(format!("Password is too weak, Warning: {}", warning)),
                None => {
                    return Err(format!(
                        "Password is too weak, Suggestion: {}",
                        data.suggestions()[0]
                    ))
                }
            },
            None => return Err("Password is too weak".to_string()),
        }
    }

    let hashed_password = hash(password, 12).unwrap();
    let user_id = nanoid!(40);
    let result = match conn.execute(
    r#"INSERT INTO lunar_buffxnte_psu.users(
      id, email, username, password, role_id, last_login, status, remember_token, created_at, updated_at, avatar)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);"#,
    &[
        &user_id,
        &email,
        &username,
        &hashed_password,
        &(2 as i64),
        &chrono::Utc::now(),
        &"Active",
        &nanoid!(30),
        &chrono::Utc::now(),
        &chrono::Utc::now(),
        &"https://cdn.psu.dev/profile.jpg"
    ],
) {
    Ok(data) => data,
    Err(err) => {
        println!("{}", err);
        return Err(String::from("Something went wrong creating the user"));
    }
};

    match regenerate_api_key(&user_id, conn) {
        Ok(_data) => (),
        Err(err) => {
            println!("ERROR: {}", err);
            return Err(String::from("Something went wrong creating the user"));
        }
    };

    return Ok(String::from("Successfully Created User"));
}

#[derive(Deserialize, Serialize)]
struct DiscordTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
    refresh_token: String,
    scope: String,
}

#[derive(Deserialize, Serialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub mfa_enabled: bool,
    pub verified: bool,
    pub email: Option<String>,
}

pub fn unlink_discord(user_id: &String, conn: MainPGDatabase) -> Result<String, String> {
    match conn.execute("UPDATE lunar_buffxnte_psu.users SET discord_id=NULL, discord_username=NULL, discord_avatar=NULL WHERE id = $1;", 
  &[&user_id])
{
  Ok(_data) => Ok("SUCCESS".to_string()),
  Err(err) => {
    println!("SQL ERROR: {}", err);
    return Err("Something went wrong unlinking this discord account. Try again later.".to_string())
  }
}
}

pub fn link_discord(
    user_id: &String,
    token: &String,
    conn: MainPGDatabase,
) -> Result<DiscordUser, String> {
    let data = ureq::post("https://discord.com/api/oauth2/token").send_form(&[
        ("client_id", &std::env::var("DISCORD_ID").unwrap()),
        ("client_secret", &std::env::var("DISCORD_SECRET").unwrap()),
        ("grant_type", "authorization_code"),
        ("code", &token),
        (
            "redirect_uri",
            "https://psu.dev/linkDiscord",
        ),
        ("scope", "identify email guilds.join"),
    ]);

    // We got a response with the token. Now handle other things...
    let token_response = match data.into_json_deserialize::<DiscordTokenResponse>() {
        Ok(data) => data,
        Err(err) => {
            println!("DISCORD ERROR: {}", err);
            return Err(
                "Something went wrong linking this discord account. Try again later.".to_string(),
            );
        }
    };

    let current_user_response = ureq::get("https://discord.com/api/users/@me")
        .set(
            "Authorization",
            &format!("Bearer {}", &token_response.access_token),
        )
        .call();

    let mut current_user = match current_user_response.into_json_deserialize::<DiscordUser>() {
        Ok(data) => data,
        Err(err) => {
            println!("DISCORD ERROR: {}", err);
            return Err(
                "Something went wrong linking this discord account. Try again later.".to_string(),
            );
        }
    };

    // Check if user has premium
    let hasPremium = has_premium(user_id, &conn);

    let mut roles: Vec<String> = Default::default();

    let computed_avatar = match &current_user.avatar {
        Some(data) => format!(
            "https://cdn.discordapp.com/avatars/{}/{}.png",
            &current_user.id, data
        ),
        None => "https://cdn.discordapp.com/embed/avatars/0.png".to_string(),
    };

    roles.push("781621726123393056".to_string());

    match hasPremium {
        Some(_data) => {
            roles.push("781621719869685780".to_string());
        }
        None => (),
    };

    // Now add the user to the discord server. No need to handle as we simply don't care if they actually join.
    let add_user_response = ureq::put(&format!(
        "https://discord.com/api/guilds/781613878407725077/members/{}",
        &current_user.id
    ))
    .set(
        "Authorization",
        &format!("Bot {}", std::env::var("DISCORD_BOTTOKEN").unwrap()),
    )
    .send_json(serde_json::json!({
      "access_token": &token_response.access_token,
      "roles": roles
    }));

    match conn.execute("UPDATE lunar_buffxnte_psu.users SET discord_id=$1, discord_username=$2, discord_avatar=$3 WHERE id = $4;", 
    &[
    &current_user.id,
    &format!("{}#{}",current_user.username, current_user.discriminator),
    &computed_avatar,
    &user_id])
  {
    Ok(data) => data,
    Err(err) => {
      println!("SQL ERROR: {}", err);
      return Err("Something went wrong linking this discord account. Try again later.".to_string())
    }
  };

    return Ok(current_user);
}

pub fn create_session(
    user_id: String,
    ip: String,
    user_agent: String,
    conn: MainPGDatabase,
) -> Result<String, String> {
    let id = nanoid!(30);

    let _result = match conn.execute(
        r#"INSERT INTO lunar_buffxnte_psu.sessions(
    id, user_id, ip_address, user_agent, payload, last_activity)
    VALUES ($1, $2, $3, $4, $5, $6);"#,
        &[
            &id,
            &user_id,
            &ip,
            &user_agent,
            &id,
            &(SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64),
        ],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err(String::from("Something went wrong creating the session"));
        }
    };

    Ok(id)
}

pub fn login(
    username: &String,
    password: &String,
    conn: MainPGDatabase,
    ip_addr: String,
    user_agent: String,
) -> Result<String, String> {
    let rows_recieved: Rows = match conn.query(
        r#"SELECT * FROM lunar_buffxnte_psu.users WHERE username = $1 OR email = $1 ORDER BY id ASC LIMIT 1"#,
        &[&username],
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", err);
            return Err(String::from("ERR_INTERNAL_ERR"));
        }
    };

    if rows_recieved.len() <= 0 {
        return Err(String::from("ERR_INVALID_CRED"));
    }

    let user = rows_recieved.get(0);

    let password_hash: String = user.get("password");

    if !verify(&password, &password_hash).unwrap() {
        return Err(String::from("ERR_INVALID_CRED"));
    }

    // Generate us a session.
    match create_session(user.get("ID"), ip_addr, user_agent, conn) {
        Ok(token) => return Ok(token),
        Err(err) => return Err(err),
    }
}
