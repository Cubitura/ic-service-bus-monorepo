use uuid_by_string::generate_uuid::{generate_uuid};
use types::{
    Subscriber, Subscribers,
};

use agent::{
    subscribe,
    subscriptions,
    subscription,
    unsubscribe
};

mod types;
mod agent;


/******************************************************/
//
//  SUBSCRIPTION MANAGEMENT
//
/******************************************************/

#[ic_cdk_macros::update]
async fn set_subscription(topic_id: String, callback: String) -> Result<String, String> {

    let result = subscribe(topic_id, callback).await;

    if result.is_ok() {
        Ok(result?)
    } else {
        Err("Could not register subscriber".to_string())
    }
}

#[ic_cdk_macros::update]
async fn unset_subscription(subscription_id: String) -> Result<String, String> {
    let result = unsubscribe(subscription_id).await;

    if result.is_ok() {
        Ok(result?)
    } else {
        Err("Could not unregister subscriber".to_string())
    }
}

#[ic_cdk_macros::update]
async fn get_subscriptions() -> Vec<Subscribers> {
    subscriptions().await
}

#[ic_cdk_macros::update]
async fn get_subscription(subscription_id: String) -> Subscriber {
    subscription(subscription_id).await
}

/******************************************************/
//
//  MISC
//
/******************************************************/

#[ic_cdk_macros::query]
fn hello(name: String) -> String {
    format!("Hello there, {}! This is an example greeting returned from a Rust backend canister!", name)  
}

#[ic_cdk_macros::query]
fn mycallback(val: String) -> Result<String, String> {

    ic_cdk::print(format!("mycallback: {}", val.to_string()));

    Ok("qwe".to_string())
}

/******************************************************/
//
//  UTILITY
//
/******************************************************/

#[ic_cdk_macros::query]
pub fn create_uuid() -> String {
    let ts = ic_cdk::api::time().to_string();
    generate_uuid(&ts)
}

