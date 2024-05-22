use candid::{CandidType, Decode, Encode};
use serde::{Deserialize, Serialize};
use ic_stable_structures::{Storable, storable::Bound};
use std::{borrow::Cow};

const MAX_VALUE_SIZE: u32 = 1024;


/******************************************************/
//
//  GENERAL PURPOSE
//
/******************************************************/



/******************************************************/
//
//  STRUCTS
//
/******************************************************/

// QUEUE ///////////////////////////////////////////
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct Message {
    pub topic: String,
    pub value: String,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct InitArgs {
    pub registry_canister: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct Subscribers {
    pub id: String,
    pub canister_id: String,
    pub callback: String,
    pub name: String,
    pub description: String,
    pub topic: String,
    pub namespace: String,
    pub active: bool,
}

#[derive(CandidType, Deserialize)]
pub struct Topics {
    pub id: String,
    pub name: String,
    pub description: String,
    pub namespaces: Vec<String>,
    pub active: bool,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct CanisterIds {
    pub ids: Vec<String>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct Idcache {
    pub ids: Vec<String>,
    pub topic: String,
    pub timestamp: u64,
}



#[derive(CandidType, Deserialize, Serialize)]
pub struct SubscriberCache {
    pub id: String,
    pub canister_id: String,
    pub callback: String,
    pub name: String,
    pub description: String,
    pub topic: String,
    pub topic_name: String,
    pub namespace: String,
    pub active: bool,
    pub timestamp: u64,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct CanisterSettings {
    pub canister_id: String,
}




/******************************************************/
//
//  STORABLES
//
/******************************************************/

// QUEUE ///////////////////////////////////////////
impl Storable for CanisterIds {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

impl Storable for Idcache {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

impl Storable for SubscriberCache {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

impl Storable for CanisterSettings {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}




