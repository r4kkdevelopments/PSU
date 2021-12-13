use core::panic;

use chrono::format;
use rocket::response::status::Custom;
use rocket::{http::Status, Data, State};
use rocket_contrib::json::{Json, JsonValue};
use serde::Deserialize;
use stripe;

use crate::{
    modules::{account_services, paypal, stripe_additions},
    MainPGDatabase,
};
#[derive(Deserialize)]
pub struct CreateOrderIDRequest {
    pub token: String,
    pub orderType: i8,
}

#[derive(Deserialize)]
pub struct GetOrderIDRequest {
    pub token: String,
    pub orderID: String,
    pub orderType: i8,
}

#[post(
    "/payments/paypal/createOrderID",
    format = "json",
    data = "<request_data>"
)]
pub fn paypal_create_order_id(
    conn: MainPGDatabase,
    shared_state: State<crate::SharedState>,
    request_data: Json<CreateOrderIDRequest>,
) -> Result<JsonValue, Custom<JsonValue>> {
    match account_services::is_authenticated(&request_data.token, &conn) {
        Ok(string) => string,
        Err(_err) => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": "ERR_INVALID_AUTH"
                }),
            ));
        }
    };

    let order_amount = match request_data.orderType {
        0 => "6.49",
        1 => "29.99",
        2 => "74.99",
        _ => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": "ERR_INVALID_ID"
                }),
            ))
        }
    };

    let lock = shared_state
        .paypalToken
        .lock()
        .expect("ASYNC MUTEX ERROR!!!! EJECTING TO PREVENT POISONING SHARED STATE!");
    let orderID = match paypal::create_order(lock.as_str(), order_amount.to_string()) {
        Ok(data) => data,
        Err(err) => {
            print!("{}", err);
            return Err(Custom(
                Status::InternalServerError,
                json!({
                  "success": false
                }),
            ));
        }
    };
    Ok(json!({"success": true, "data": orderID}))
}

#[post(
    "/payments/paypal/getOrderID",
    format = "json",
    data = "<request_data>"
)]
pub fn paypal_get_order_id(
    conn: MainPGDatabase,
    shared_state: State<crate::SharedState>,
    request_data: Json<GetOrderIDRequest>,
) -> Result<JsonValue, Custom<JsonValue>> {
    let user_id = match account_services::is_authenticated(&request_data.token, &conn) {
        Ok(string) => string,
        Err(_err) => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": "ERR_INVALID_AUTH"
                }),
            ));
        }
    };

    let order_amount = match request_data.orderType {
        0 => "6.49",
        1 => "29.99",
        2 => "74.99",
        _ => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": "ERR_INVALID_ID"
                }),
            ))
        }
    };

    let lock = shared_state
        .paypalToken
        .lock()
        .expect("ASYNC MUTEX ERROR!!!! EJECTING TO PREVENT POISONING SHARED STATE!");

    let data = match paypal::get_order(
        &lock,
        &request_data.orderID,
        &user_id,
        &request_data.orderType,
        &conn,
    ) {
        Ok(data) => data,
        Err(err) => {
            print!("{}", err);
            return Err(Custom(
                Status::InternalServerError,
                json!({
                  "success": false
                }),
            ));
        }
    };

    if order_amount != data.purchase_units[0].amount.value {
        return Err(Custom(
            Status::InternalServerError,
            json!({
              "success": false,
              "message": "Order Amount doesn't sync with orderID"
            }),
        ));
    }

    Ok(json!({"success": true, "status": data.status}))
}

#[post(
    "/payments/stripe/createOrderID",
    format = "json",
    data = "<request_data>"
)]
pub fn stripe_create_order_id(
    conn: MainPGDatabase,
    shared_state: State<crate::SharedState>,
    request_data: Json<CreateOrderIDRequest>,
) -> Result<JsonValue, Custom<JsonValue>> {
    let user_id = match account_services::is_authenticated(&request_data.token, &conn) {
        Ok(string) => string,
        Err(_err) => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": "ERR_INVALID_AUTH"
                }),
            ));
        }
    };

    let priceID = match request_data.orderType {
        0 => "price_1IK90rEnq0tzcOdN219ShCgm",
        1 => "price_1IK90nEnq0tzcOdNWXozTZ4p",
        2 => "price_1J8yNzEnq0tzcOdNN4pePZMs",
        _ => {
            return Err(Custom(
                Status::BadRequest,
                json!({
                  "success": false,
                  "message": "ERR_INVALID_ID"
                }),
            ))
        }
    };

    let stripeClient: stripe::Client = stripe::Client::new(dotenv::var("STRIPE_KEY").unwrap());

    let mut params = stripe_additions::CreateSession::new();

    params.mode = Some(stripe::CheckoutSessionMode::Payment);
    params.line_items = Some(vec![stripe_additions::CheckoutSessionItem {
        price: Some(priceID.to_string()),
        quantity: 1,
    }]);
    params.success_url = Some("https://psu.dev/purchaseSuccess".to_string());
    params.cancel_url = Some("https://psu.dev/purchasePremium".to_string());
    params.payment_method_types = Some(vec![stripe::PaymentMethodType::Card]);
    params.client_reference_id = Some(user_id);

    let orderID = match stripe_additions::create_checkout(&stripeClient, params) {
        Ok(data) => data,
        Err(err) => {
            return Err(Custom(
                Status::InternalServerError,
                json!({
                  "success": false,
                  "message": "Something went wrong with Stripe. Please contact Support."
                }),
            ))
        }
    };

    Ok(json!({"success": true, "data": orderID.id }))
}

use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;

pub struct StripeHeader(String);

#[derive(Debug)]
pub enum ApiUserAgentError {
    Missing,
}

impl<'a, 'r> FromRequest<'a, 'r> for StripeHeader {
    type Error = ApiUserAgentError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let token = request.headers().get_one("stripe-signature");
        match token {
            Some(token) => {
                // check validity
                Outcome::Success(StripeHeader(token.to_string()))
            }
            None => Outcome::Failure((Status::Unauthorized, ApiUserAgentError::Missing)),
        }
    }
}

use std::io::Read;
#[post(
    "/private/webhooks/16okicruFoErFYDXXNp6DQsoLEnM50dkPGyYNsWH",
    format = "json",
    data = "<request_data>"
)]
pub fn stripe_handle_webhook(
    conn: MainPGDatabase,
    shared_state: State<crate::SharedState>,
    request_data: Data,
    stripe_header: StripeHeader,
) -> Result<String, Custom<String>> {
    let contents = {
        let mut buf = String::new();
        request_data.open().read_to_string(&mut buf).unwrap();
        buf
    };

    let webHookData = match stripe::Webhook::construct_event(
        &contents,
        &stripe_header.0,
        &dotenv::var("STIPE_WEBHOOK_KEY").unwrap(),
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("WEBHOOK ERROR: {}", err);
            return Err(Custom(
                Status::InternalServerError,
                "Something went wrong processing the WebHook".to_string(),
            ));
        }
    };

    match stripe_additions::stripe_event_handler(webHookData, &conn) {
        Ok(_data) => return Ok("SUCCESS".to_string()),
        Err(err) => return Err(Custom(Status::InternalServerError, err)),
    }
}

struct Signature {
    t: i64,
    v1: String,
}

fn parse(raw: &str) -> Result<Signature, &str> {
    use std::collections::HashMap;
    let headers: HashMap<&str, &str> = raw
        .split(',')
        .map(|header| {
            let mut key_and_value = header.split('=');
            let key = key_and_value.next();
            let value = key_and_value.next();
            (key, value)
        })
        .filter_map(|(key, value)| match (key, value) {
            (Some(key), Some(value)) => Some((key, value)),
            _ => None,
        })
        .collect();
    let t = headers.get("t").ok_or("Bad Sig, FAILED 1")?;
    let v1: String = headers
        .get("v1")
        .ok_or("Bad Sig, FAILED 2")
        .unwrap()
        .to_string();
    Ok(Signature {
        t: t.parse::<i64>().unwrap(),
        v1,
    })
}
