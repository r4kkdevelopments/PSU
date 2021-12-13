use base64::encode;
use core::panic;
use serde::Deserialize;
use dotenv::dotenv;

use crate::{modules::account_services, MainPGDatabase};

pub mod schemas;

#[derive(Deserialize)]
struct clientIDReponse {
    pub scope: String,
    pub access_token: String,
    pub token_type: String,
    pub app_id: String,
    pub expires_in: u32,
    pub nonce: String,
}
#[derive(Deserialize)]
pub struct PayPalOrderLinks {
    href: String,
    rel: String,
    method: String,
}

#[derive(Deserialize)]
pub struct PayPalOrder {
    pub id: String,
    pub status: String,
    pub links: Vec<PayPalOrderLinks>,
}

#[derive(Deserialize)]
pub struct PayPalError {
    pub error: String,
    pub error_description: String,
}

pub fn handle_paypal_error(error: serde_json::Value) -> PayPalError {
    let parsedData: PayPalError = match serde_json::value::from_value(error.to_owned()) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", error);
            PayPalError {
                error: "GENERIC_ERROR".to_string(),
                error_description: "Failed to decode error message. Check Logs".to_string(),
            }
        }
    };

    parsedData
}

pub fn get_client_id(clientID: &String, secret: &String) -> String {
    let resp = ureq::post("https://api-m.paypal.com/v1/oauth2/token")
        .set(
            "Authorization",
            &format!("Basic {}", encode(format!("{}:{}", clientID, secret))),
        )
        .send_form(&[("grant_type", "client_credentials")]);

    let test = &resp.into_json();

    let data = match test {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", &test);
            panic!("FATAL ERROR! Failed to authenticate with paypal. Either ClientID or Secret was Invalid.");
        }
    };

    let parsedData: clientIDReponse = match serde_json::value::from_value(data.to_owned()) {
        Ok(data) => data,
        Err(err) => {
            let error = handle_paypal_error(data.to_owned());
            panic!(
                "Failed to authenticate with paypal! Error: {}, Error Desc: {}",
                error.error, error.error_description
            );
        }
    };

    println!(
        "Successfully authenticated as app_id: {}",
        parsedData.app_id
    );

    return parsedData.access_token;
}

pub fn new(clientID: String, secret: String) -> String {
    println!("PayPal Module Initialising...");
    println!(
        "Attempting PayPal authentication as Client ID: {}",
        &clientID
    );
    // Attempt authorisation
    let token = get_client_id(&clientID, &secret);

    return token;
}

pub fn get_order(
    current_token: &str,
    order_id: &String,
    user_id: &String,
    orderType: &i8,
    conn: &MainPGDatabase,
) -> Result<schemas::OrderSchema, String> {
    // Get new token (TEMPORARY)
    let token = get_client_id(&dotenv!("PAYPAL_ID").to_string(), &dotenv!("PAYPAL_SECRET").to_string());

    let response = ureq::get(&format!(
        "https://api-m.paypal.com/v2/checkout/orders/{}",
        order_id
    ))
    .set("Authorization", &format!("Bearer {}", &token))
    .call();

    let responseJSON = &response.into_json();

    let data = match responseJSON {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", &responseJSON);
            panic!("FATAL ERROR! Failed to parse paypal struct!");
        }
    };

    let parsedData: schemas::OrderSchema = match serde_json::value::from_value(data.to_owned()) {
        Ok(data) => data,
        Err(err) => {
            let error = handle_paypal_error(data.to_owned());
            panic!(
                "Failed to parse paypal struct! Error: {}, Error Desc: {}",
                error.error, error.error_description
            );
        }
    };

    match account_services::add_premium(user_id, order_id, &"PayPal".to_string(), orderType, conn) {
        Ok(data) => data,
        Err(err) => return Err(err),
    };

    Ok(parsedData)
}

pub fn create_order(currentToken: &str, price: String) -> Result<String, String> {
    // Get new token (TEMPORARY)
    let token = get_client_id(&dotenv!("PAYPAL_ID").to_string(), &dotenv!("PAYPAL_SECRET").to_string());

    // Construct our query and send it
    let response = ureq::post("https://api-m.paypal.com/v2/checkout/orders")
        .set("Authorization", &format!("Bearer {}", &token))
        .send_json(ureq::json!({
          "intent": "CAPTURE",
          "application_context": {
              "brand_name": "PSU",
              "shipping_preference": "NO_SHIPPING",
              "user_action": "PAY_NOW",
              "return_url": "https://psu.dev/"
          },
          "purchase_units": [
            {
              "description": "You agree that you are receiving Premium which is given to you on our website. Refunds only allowed if product isn't delivered.",
              "amount": {
                "currency_code": "USD",
                "value": price
              }
            }
          ]
        }));

    let responseJSON = &response.into_json();

    let data = match responseJSON {
        Ok(data) => data,
        Err(err) => {
            println!("{:?}", &responseJSON);
            panic!("FATAL ERROR! Failed to authenticate with paypal. Either ClientID or Secret was Invalid.");
        }
    };

    let parsedData: PayPalOrder = match serde_json::value::from_value(data.to_owned()) {
        Ok(data) => data,
        Err(err) => {
            let error = handle_paypal_error(data.to_owned());
            panic!(
                "Failed to authenticate with paypal! Error: {}, Error Desc: {}",
                error.error, error.error_description
            );
        }
    };

    Ok(parsedData.id)
}
