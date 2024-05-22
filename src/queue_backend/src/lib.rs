use std::process::Command;
use ic_cdk::init;
use std::vec::Vec;
use std::{borrow::Cow};
use candid::{CandidType, Decode, Encode, Principal};
use std::collections::VecDeque;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use ic_stable_structures::{
    Storable, storable::Bound,
    DefaultMemoryImpl, 
    BTreeMap,
    StableBTreeMap,
    memory_manager::MemoryId,
    memory_manager::MemoryManager,
    memory_manager::VirtualMemory,
};
use ic_cdk_timers::TimerId;
use std::{
    cell::RefCell,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use types::{
    Message, Subscribers, Topics, 
    CanisterIds, Idcache,
    SubscriberCache, CanisterSettings,

};

mod types;

const MAX_CHUNK_SIZE: usize = 250;
const CACHE_TTL_NS: u64 = 60000000000;

static INITIAL_CANISTER_BALANCE: AtomicU64 = AtomicU64::new(0);
static CYCLES_USED: AtomicU64 = AtomicU64::new(0);
static MIN_INTERVAL_SECS: u64 = 10;

static mut LOCKCD: RwLock<VecDeque::<String>> = RwLock::new(VecDeque::<String>::new());


/******************************************************/
//
//  INIT
//
/******************************************************/

#[init]
pub fn init() -> () {
    start_with_interval_secs(MIN_INTERVAL_SECS);
}
 
#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    start_with_interval_secs(MIN_INTERVAL_SECS);
}


/******************************************************/
//
//  MEMORY MANAGER
//
/******************************************************/

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEM_WHITELIST: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static WHITELIST: RefCell<StableBTreeMap<String, CanisterIds, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEM_WHITELIST.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );    

    static MEM_SUBSCRIBER_CACHE: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static SUBSCRIBER_CACHE: RefCell<BTreeMap<String, Idcache, Memory>> = RefCell::new(
        BTreeMap::init(
            MEM_SUBSCRIBER_CACHE.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );       

    static MEM_SUBSCRIBER_DATA_CACHE: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static SUBSCRIBER_DATA_CACHE: RefCell<BTreeMap<String, SubscriberCache, Memory>> = RefCell::new(
        BTreeMap::init(
            MEM_SUBSCRIBER_DATA_CACHE.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

    static MEM_CANISTER_SETTINGS: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static CANISTER_SETTINGS: RefCell<BTreeMap<String, CanisterSettings, Memory>> = RefCell::new(
        BTreeMap::init(
            MEM_CANISTER_SETTINGS.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );

    static COUNTER: RefCell<u32> = RefCell::new(0);
    static TIMER_IDS: RefCell<Vec<TimerId>> = RefCell::new(Vec::new());
}


/******************************************************/
//
//  FIFO BUFFER
//
//  fifo_producer       Inserts new messages in the queue
//  fifo_consumer       Processes messages in chunks
//  fifo_buffer_size    Get the current queue size
//  fifo_buffer_empty   Clear the queue
//  serialize_message   Serialize message before storing in queue
//  deserialize_message Deserialize messages stored in the queue
//
/******************************************************/

#[ic_cdk_macros::update]
fn fifo_producer(msg: Message) -> Result<String, String> {
    if msg.topic.len() == 0 || msg.value.len() == 0 {
        return Err("Message is invalid. Message is missing topic and/or value".to_string())
    }
    
    let do_insert = || -> Result<String, String> {
        let msg_str: String = serialize_message(msg);
        unsafe {
            let mut fifo_ref = LOCKCD.write().unwrap();
            fifo_ref.push_back(msg_str);
            drop(fifo_ref);
        }
        Ok("Success: Element was inserted in the queue1".to_string())
    };

    if let Err(_err) = do_insert() {
        return Err("Failed to perform necessary steps".to_string())
    } else {
        return Ok("Success: Element was inserted in the queue2".to_string())
    }
}

fn fifo_consumer() {    
    let do_insert = || -> Result<String, String> {
        unsafe {
            let mut fifo_ref = LOCKCD.write().unwrap();
            let mut chunk_size: usize = MAX_CHUNK_SIZE;

            if fifo_ref.len() < MAX_CHUNK_SIZE {
                chunk_size = fifo_ref.len();
            }

            if chunk_size <= 0 {
                return Ok("Success: There are no messages to process".to_string())
            }

            let drained = fifo_ref.drain(0..chunk_size).collect::<VecDeque<String>>();
            
            for msg in drained.iter() {
                ic_cdk::spawn( 
                    route_message(deserialize_message(msg.clone()))
                );
            }

            drop(fifo_ref);
        }
        Ok("Success: Messages were processed successfully".to_string())
    };

    if let Ok(_res) = do_insert() {
        ic_cdk::print(_res.to_string());
    } else {
        ic_cdk::print("Success: Messages were processed successfully".to_string());
    }
}

#[ic_cdk_macros::query]
fn fifo_buffer_size() -> usize {
    unsafe {
        let fifo_ref = LOCKCD.write().unwrap();
        fifo_ref.len() 
    }
}

#[ic_cdk_macros::update]
fn fifo_buffer_empty() -> () {
    unsafe {
        let mut deque = LOCKCD.write().unwrap();
        let _ = deque.drain(..).collect::<VecDeque<String>>();
    }
}

#[ic_cdk_macros::query]
fn serialize_message(msg: Message) -> String {
    let json_string: String = serde_json::to_string(&msg).unwrap();
    json_string
}

#[ic_cdk_macros::query]
fn deserialize_message(msg: String) -> Message {
    let message: Message = serde_json::from_str(&msg).unwrap();
    message
}


/******************************************************/
//
//  WHITELIST
//
//  whitelist_register          Whitelist canister ID
//  whitelist_unregister        Remove whitelist of canister ID
//  whitelist_lookup            Lookup canister ID permission
//  whitelist_canister_check    Check is a canister is allowed 
//                              to send a message
//
/******************************************************/

#[ic_cdk_macros::update]
fn whitelist_register(topic: String, canister_id: String) -> () {
    let cid = WHITELIST.with(|p| p.borrow().get(&topic));
    
    if cid.is_none() {
        let mut ids = Vec::<String>::new();
        ids.push(canister_id);

        let canister_ids = CanisterIds {
            ids: ids.clone()
        };

        WHITELIST.with(|p| p.borrow_mut().insert(topic, canister_ids));
    } else {
        let mut ids =  cid.unwrap().ids;
        ids.push(canister_id.to_string());

        let canister_ids = CanisterIds {
            ids: ids.clone()
        };
        WHITELIST.with(|p| p.borrow_mut().insert(topic.clone(), canister_ids));
    }
}

#[ic_cdk_macros::update]
fn whitelist_unregister(topic: String, canisterId: String) -> Result<String, String> {
    let cid = WHITELIST.with(|p| p.borrow().get(&topic.clone()));
    
    if cid.is_some() {
        let mut ids: Vec<String> = cid.unwrap().ids;
        ids.retain(|x| *x != canisterId);
        
        WHITELIST.with(|p| p.borrow_mut().insert(topic.clone(), CanisterIds { ids: ids.clone() }));
        Ok("The canister id was removed from the topic whitelist".to_string())
    } else {
        Err("The canister id was not found for this topic".to_string())
    }
}

#[ic_cdk_macros::query]
fn whitelist_lookup(topic: String) -> Vec<String> {
    let cid = WHITELIST.with(|p| p.borrow().get(&topic));
    
    if cid.is_some() {
        cid.unwrap().ids
    } else {        
        Vec::new()
    }
}

#[ic_cdk_macros::query]
fn whitelist_canister_check(topic: String, canisterId: String) -> Result<String, String> {
    let cid = WHITELIST.with(|p| p.borrow().get(&topic));
    
    if cid.is_some() {
        let ids: Vec<String> = cid.unwrap().ids;

        if ids.iter().any(|i| i == &canisterId) {
            Ok("Success: The canister id is whitelisted for this topic".to_string())  
        } else {
            Err("Fail: The canister id is not whitelisted for this topic".to_string())
        }

    } else {        
        Err("Fail: The canister id is not whitelisted for this topic".to_string())
    }
}


/******************************************************/
//
//  INTAKE
//
//  intake      The main message intake function, it checks
//              if the canister sending the request is
//              whitelisted 
//
/******************************************************/

#[ic_cdk_macros::update]
pub async fn intake(msg: Message) -> Result<String, String> {
    let subscriber_principal_id = ic_cdk::caller();
    let Message { ref topic, ref value } = msg;
    let whitelist_check = whitelist_canister_check(topic.clone(), subscriber_principal_id.to_string());

    if whitelist_check.is_ok() {
        let inject = fifo_producer(Message {
            topic: topic.clone(),
            value: value.clone(),
        });

        let inject = fifo_producer(msg);

        if inject.is_ok() {
            Ok("Success: The message has been pushed to the queue".to_string())
        } else {
            Err("Fail: The message was rejected by the queue".to_string())
        }
    } else {
        Err("Fail: The sending canister is not whitelisted".to_string())
    }
}
    
    
/******************************************************/
//
//  TIMER
//
//  start_with_interval_secs    Starts a timer, which works as
//                              a periodic check to see if there
//                              are messages to process
//  
//
/******************************************************/

pub fn track_cycles_used() {
    let current_canister_balance = ic_cdk::api::canister_balance();
    INITIAL_CANISTER_BALANCE.fetch_max(current_canister_balance, Ordering::Relaxed);
    let cycles_used = INITIAL_CANISTER_BALANCE.load(Ordering::Relaxed) - current_canister_balance;
    CYCLES_USED.store(cycles_used, Ordering::Relaxed);
}

#[ic_cdk_macros::query]
pub fn counter() -> u32 {
    COUNTER.with(|counter| *counter.borrow())
}

#[ic_cdk_macros::update]
pub fn start_with_interval_secs(secs: u64) {
    let secs = Duration::from_secs(secs);
    let timer_id = ic_cdk_timers::set_timer_interval(secs, fifo_consumer);
    TIMER_IDS.with(|timer_ids| timer_ids.borrow_mut().push(timer_id));
}

#[ic_cdk_macros::update]
pub fn stop() {
    TIMER_IDS.with(|timer_ids| {
        if let Some(timer_id) = timer_ids.borrow_mut().pop() {
            ic_cdk_timers::clear_timer(timer_id);
        }
    });
}

#[ic_cdk_macros::query]
pub fn cycles_used() -> u64 {
    CYCLES_USED.load(Ordering::Relaxed)
}


/******************************************************/
//
//  MESSAGE ROUTER
//
/******************************************************/

#[ic_cdk_macros::update]
async fn route_message(message: Message) -> () {
    for subscriber in cache_subscribers(message.clone().topic).await.iter() {
        let subscriber_data = cache_subscriber_data(subscriber.to_string()).await.unwrap();
        
        ic_cdk::spawn( 
            route_message_execute(subscriber_data.canister_id.to_string(), subscriber_data.callback.to_string(), message.clone().value)
        );
    }
}

#[ic_cdk_macros::query]
async fn route_message_execute(canister_id: String, callback: String, val: String) -> () {
    let transfer_result: (Result<String, String>, ) = ic_cdk::call(Principal::from_text(&canister_id.to_string()).expect("Could not decode the principal."), &callback.to_string(), (&val.to_string(),)).await.unwrap();
}



/******************************************************/
//
//  CACHE
//
/******************************************************/

#[ic_cdk_macros::query]
async fn cache_subscribers(topic: String) -> Vec<String> {
    let topic_cache = SUBSCRIBER_CACHE.with(|p| p.borrow().get(&topic));

    if topic_cache.is_some() {
        topic_cache.unwrap().ids
    } else {
        Vec::new()
    }
}

#[ic_cdk_macros::query]
async fn cache_subscriber_data(topic_id: String) -> Option<SubscriberCache> {
    let subscriber_data_cache = SUBSCRIBER_DATA_CACHE.with(|p| p.borrow().get(&topic_id));

    if subscriber_data_cache.is_some() {
        Some(subscriber_data_cache.unwrap())
    } else {
        None
    }
}

#[ic_cdk_macros::update]
async fn cache_subscribers_fetch() -> () {
    let registry_canister = canister_settings_get("registry_backend".to_string()).unwrap().canister_id;
    let topics: (Vec<Topics>, ) = ic_cdk::call(Principal::from_text(registry_canister).expect("Could not decode the principal."), "topics", ()).await.unwrap();
    let subscribers: (Vec<Subscribers>, ) = ic_cdk::call(Principal::from_text(registry_canister).expect("Could not decode the principal."), "subscribers", ()).await.unwrap();
    let mut ids: Vec<String> = Vec::new();

    for i in subscribers.0.iter() {
        let topic_id = &i.topic;

        let index: usize = topics.0
            .iter()
            .position(|x| x.id == topic_id.to_string()).unwrap();

        let topic_name = topics.0.get(index).unwrap().name.to_string();
        let mut topic_cache = SUBSCRIBER_CACHE.with(|p| p.borrow().get(&topic_name));
        let ts = ic_cdk::api::time();

        let mut id_insert = Idcache {
            ids: Vec::new(),
            topic: topic_name.clone(),
            timestamp: ts,
        };

        let subscriber_insert = SubscriberCache {
            id: i.id.to_string(),
            canister_id: i.canister_id.to_string(),
            callback: i.callback.to_string(),
            name: i.name.to_string(),
            description: i.description.to_string(),
            topic: i.topic.to_string(),
            topic_name: topic_name.clone(),
            namespace: i.namespace.to_string(),
            active: i.active,
            timestamp: ts,
        };

        if topic_cache.is_some() {
            id_insert.ids = topic_cache.unwrap().ids;

            if !id_insert.ids.contains(&i.id) {
                id_insert.ids.push(i.id.to_string());
                SUBSCRIBER_DATA_CACHE.with(|p| p.borrow_mut().insert(i.id.to_string(), subscriber_insert));
            }

            SUBSCRIBER_CACHE.with(|p| p.borrow_mut().insert(topic_name.clone(), id_insert));
        } else {
            id_insert.ids.push(i.id.to_string());
            SUBSCRIBER_CACHE.with(|p| p.borrow_mut().insert(topic_name.clone(), id_insert));
            SUBSCRIBER_DATA_CACHE.with(|p| p.borrow_mut().insert(i.id.to_string(), subscriber_insert));
        }
    }
}

#[ic_cdk_macros::update]
fn cache_subscribers_clear() -> () {
    SUBSCRIBER_CACHE.with(|p| p.borrow_mut().clear_new());
    SUBSCRIBER_DATA_CACHE.with(|p| p.borrow_mut().clear_new());
}


/******************************************************/
//
//  CACHE
//
/******************************************************/

#[ic_cdk_macros::update]
pub async fn canister_settings_store(canister_name: String, canister_id: String) -> () {
    let _ = CANISTER_SETTINGS.with(|p| p.borrow_mut().insert(canister_name.to_string(), CanisterSettings { canister_id: canister_id.to_string() }));
}

#[ic_cdk_macros::query]
pub fn canister_settings_get(canister_name: String) -> Option<CanisterSettings> {
    CANISTER_SETTINGS.with(|p| p.borrow().get(&canister_name))
}


