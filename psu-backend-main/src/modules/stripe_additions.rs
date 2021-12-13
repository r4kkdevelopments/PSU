extern crate stripe;
use crate::{modules::account_services, MainPGDatabase};
use colored::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CheckoutSessionItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    pub quantity: i8,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSession {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<stripe::CheckoutSessionMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_types: Option<Vec<stripe::PaymentMethodType>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<CheckoutSessionItem>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_reference_id: Option<String>,
}

impl CreateSession {
    pub fn new() -> Self {
        CreateSession {
            success_url: Default::default(),
            cancel_url: Default::default(),
            mode: Default::default(),
            payment_method_types: Default::default(),
            line_items: Default::default(),
            client_reference_id: Default::default(),
        }
    }
}

pub fn handle_checkout_complete(data: EventData, conn: &MainPGDatabase) -> Result<String, String> {
    // here handle checkout completion
    let checkoutData = match data.object {
        stripe::EventObject::CheckoutSession(data) => data,
        _ => return Err("Something went wrong processing this data.".to_string()),
    };

    let orderType = match checkoutData.amount_total {
        649 => 0 as i8,  // One Month
        2999 => 1 as i8, // One Year
        7499 => 2 as i8, // Lifetime
        _ => return Err("Something went wrong processing this data.".to_string()),
    };

    match account_services::add_premium(
        &checkoutData.client_reference_id.unwrap(),
        &checkoutData.id.to_string(),
        &"Stripe".to_string(),
        &orderType,
        conn,
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("Error while processing stripe webhook.");
            return Err(err)
        }
    };

    println!("Success while processing stripe webhook!");
    Ok("SUCCESS".to_string())
}

pub fn create_checkout(
    client: &stripe::Client,
    params: CreateSession,
) -> stripe::Response<stripe::CheckoutSession> {
    client.post_form("/checkout/sessions", &params)
}

use stripe::{EventData, EventType::*};

pub fn stripe_event_handler(event: stripe::Event, conn: &MainPGDatabase) -> Result<String, String> {
    println!(
        "[{}] Handling WebHook Event: {:?}",
        "STRIPE".blue(),
        &event.event_type
    );
    match event.event_type {
        CheckoutSessionCompleted => handle_checkout_complete(event.data, conn),
        _ => Ok("UNIMPLEMENTED".to_string()),
    }
}
