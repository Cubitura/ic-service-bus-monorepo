use std::vec::Vec;
use candid::{CandidType};
use std::collections::VecDeque;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};


use uuid_by_string::generate_uuid::{generate_uuid};


static mut LOCKVD: RwLock<VecDeque::<String>> = RwLock::new(VecDeque::<String>::new());

/******************************************************/
//
//  TYPES
//
/******************************************************/

#[derive(CandidType, Deserialize, Serialize)]
pub struct LogEntry {
    pub log_id: String,              // Log ID
    pub log_type: u8,            // Success, Error, Warning
    pub log_origin: String,          // Calling service
    pub log_canister: String,        // Service canister ID    
    pub log_message: String,         // Success/Error message
    pub log_data: String,            // Payload consumed by the service
    pub log_timestamp: String,       // Timestamp
}

/******************************************************/
//
//  PUBLIC FUNCTION
//
/******************************************************/
#[ic_cdk_macros::update]
pub async fn log(logtype: u8, origin: String, message: String, data: String) -> Result<String, String> {
    let submitter_principal = ic_cdk::caller();
    let ts = ic_cdk::api::time().to_string();

    let entry = LogEntry {
        log_id: generate_uuid(&ts),
        log_type: logtype,
        log_origin: origin,
        log_canister: submitter_principal.to_string(),
        log_message: message,
        log_data: data,
        log_timestamp: ts,
    };

    log_inject(entry)
}

/******************************************************/
//
//  LOG BUFFER
//
/******************************************************/

#[ic_cdk_macros::update]
fn log_inject(msg: LogEntry) -> Result<String, String> {
    let do_insert = || -> Result<String, String> {
        let msg_str: String = serialize_message(msg);
        
        unsafe {
            let mut log_ref = LOCKVD.write().unwrap();
            log_ref.push_back(msg_str);
            drop(log_ref);
        }
        Ok("Success: Element was inserted in the queue1".to_string())
    };

    if let Err(_err) = do_insert() {
        return Err("Failed to perform necessary steps".to_string())
    } else {
        return Ok("Success: Element was inserted in the queue2".to_string())
    }
}

#[ic_cdk_macros::query]
fn log_range(index: usize, length: usize) -> Vec<LogEntry> {

    unsafe {
        let log_ref = LOCKVD.read().unwrap();
        let mut end: usize = index + length;

        if log_ref.len() < end + 1 { end = log_ref.len(); }

        ic_cdk::print(end.to_string());

        if log_ref.len() == 0 || log_ref.len() < index + 1 {
            return Vec::new()  
        }

        let range = log_ref.range(index..end);
        let mut range_output = Vec::new();
        
        for job in range {
            range_output.push(deserialize_message(job.clone()));
        }

        range_output
    }
}

#[ic_cdk_macros::query]
fn log_size() -> usize {
    unsafe {
        let log_ref = LOCKVD.write().unwrap();
        log_ref.len() 
    }
}

#[ic_cdk_macros::update]
fn log_empty() -> () {
    unsafe {
        let mut deque = LOCKVD.write().unwrap();
        let _ = deque.drain(..).collect::<VecDeque<String>>();
    }
}

#[ic_cdk_macros::query]
fn serialize_message(msg: LogEntry) -> String {
    let json_string: String = serde_json::to_string(&msg).unwrap();
    json_string
}

#[ic_cdk_macros::query]
fn deserialize_message(msg: String) -> LogEntry {
    let message: LogEntry = serde_json::from_str(&msg).unwrap();
    message
}


/******************************************************/
//
//  UPGRADE
//
/******************************************************/

#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade() {

}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {

}