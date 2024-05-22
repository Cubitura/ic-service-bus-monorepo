use candid::{Principal};

use ic_stable_structures::{
    DefaultMemoryImpl, 
    StableBTreeMap,
    memory_manager::MemoryId,
    memory_manager::MemoryManager,
    memory_manager::VirtualMemory,
};
use std::{cell::RefCell};
use types::{
    Namespace, Namespaces, 
    Topic, Topics, Subscriber, Subscribers, 
    ResultResponse, CallStringResponse, 
    CallSubscribersResponse, CallSubscriberResponse,
    Canisters
};

use utils::{create_uuid, get_variable_type};
use ic_cdk::print;

mod types;
mod utils;


/******************************************************/
//
//  MEMORY MANAGER
//
/******************************************************/

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEM_MAN_TOPIC: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static MAP_TOPIC: RefCell<StableBTreeMap<String, Topic, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEM_MAN_TOPIC.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static MEM_MAN_NAMESPACE: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static MAP_NAMESPACE: RefCell<StableBTreeMap<String, Namespaces, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEM_MAN_NAMESPACE.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );

    static MEM_MAN_SUBSCRIBER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static MAP_SUBSCRIBER: RefCell<StableBTreeMap<String, Subscribers, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEM_MAN_SUBSCRIBER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

    static MEM_MAN_CANISTER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static MAP_CANISTER: RefCell<StableBTreeMap<String, Canisters, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEM_MAN_CANISTER.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );
}

/******************************************************/
//
//  NAMESPACE
//
//  namespace_register              Add namespace
//  namespace_unregister            Remove namespace
//  namespace                       Get a specific namespace's details
//  namespaces                      Get all namespaces
//  namespace_register_subscriber   Add subscriber to a namespace
//  namespace_subscriber_size       Get a namespace's number of subscribers
//  namespaces_by_topic             Get namespaces of a specific topic
//  namespace_by_subscriber         Get namespace by subscriber
//  
/******************************************************/

#[ic_cdk_macros::update]
fn namespace_register(namespace: Namespace) -> Result<String, String> {
    let id = create_uuid();

    let ns = Namespaces {
        id: id.clone(),
        name: namespace.name,
        description: namespace.description,
        subscribers: namespace.subscribers,
        active: namespace.active,
    };

    let res = MAP_NAMESPACE.with(|p| p.borrow_mut().insert(id.clone(), ns));

    if get_variable_type(&res).contains("Namespace") {
        Ok(id.clone().to_string())
    } else {
        Err("Could not register the namespace".to_string())
    }   
}

#[ic_cdk_macros::update]
fn namespace_unregister(namespace_id: String) -> Result<String, String>  {
    
    if !MAP_NAMESPACE.with(|p| p.borrow().contains_key(&namespace_id)) {
        return Err("The namespace was not found".to_string());
    }

    let res = MAP_NAMESPACE.with(|p| {p.borrow_mut().remove(&namespace_id)});

    if get_variable_type(&res).contains("Namespace") {
        Ok(namespace_id.to_string())
    } else {
        Err("Could not unregister the namespace".to_string())
    }   
}

#[ic_cdk_macros::update]
fn namespace_register_subscriber(namespace_id: String, subscriber_id: String) -> Result<String, String> { 
    let mut ns: Namespaces = namespace(namespace_id.clone()).unwrap();
    ns.subscribers.push(subscriber_id.clone());

    let res = MAP_NAMESPACE.with(|p| p.borrow_mut().insert(namespace_id.clone(), ns));

    if get_variable_type(&res).contains("Namespace") {
        Ok(namespace_id.clone())
    } else {
        Err("Could not add subscriber".to_string())
    }   
}

#[ic_cdk_macros::query]
fn namespace(namespace_id: String) -> Option<Namespaces> {
    MAP_NAMESPACE.with(|p| p.borrow().get(&namespace_id))
}

#[ic_cdk_macros::query]
fn namespaces() -> Vec<Namespaces> {
    let mut namespaces: Vec<Namespaces> = Vec::new();

    MAP_NAMESPACE.with(|p| {
        for (k, v) in p.borrow().iter() {
            let nspace = Namespaces {
                id: k,
                name: v.name,
                description: v.description,
                subscribers: v.subscribers,
                active: v.active,
            };

            namespaces.push(nspace);
        }
    }); 
    namespaces   
}

#[ic_cdk_macros::query]
fn namespace_subscriber_size(namespace_id: String) -> usize {
    let namespace = namespace(namespace_id).unwrap();
    namespace.subscribers.len()
}

#[ic_cdk_macros::query]
fn namespaces_by_topic(topic_id: String) -> Vec<Namespaces> {
        let topic = topic(topic_id);
        let mut namespaces: Vec<Namespaces> = Vec::new();
    
        MAP_NAMESPACE.with(|p| {
            for (k, v) in p.borrow().iter() {
                
                if topic.namespaces.iter().any(|i| i == &k) {
                    
                    let ns = Namespaces {
                        id: k,
                        name: v.name,
                        description: v.description,
                        subscribers: v.subscribers,
                        active: v.active,
                    };
    
                    namespaces.push(ns);
                }
                
            }
        }); 
    
        namespaces
}

#[ic_cdk_macros::query]
fn namespace_by_subscriber(subscriber_id: String) -> Namespaces {
        let namespace_id = subscriber(subscriber_id.clone()).namespace;
        let ns = namespace(namespace_id.clone()).unwrap();

        Namespaces {
            id: namespace_id.to_string(),
            name: ns.name,
            description: ns.description,
            subscribers: ns.subscribers,
            active: ns.active,
        }
}

/******************************************************/
//
//  Topic
//
//  topic_register      Register topic
//  topic_unregister    Remove topic
//  topic               Lookup topic
//  topics              Get all topics
//  topic_by_name       Get topic by name
//
/******************************************************/

#[ic_cdk_macros::update]
fn topic_register(topic: Topic) -> Result<String, String> {
    let id = create_uuid();
    let res = MAP_TOPIC.with(|p| p.borrow_mut().insert(id.clone(), topic));

    if get_variable_type(&res).contains("Topic") {
        Ok(id.to_string())
    } else {
        Err("Could not register the topic".to_string())
    }   
}

#[ic_cdk_macros::update]
fn topic_unregister(topic_id: String) -> Result<String, String>  {
    
    if !MAP_TOPIC.with(|p| p.borrow().contains_key(&topic_id)) {
        return Err("The topic was not found".to_string());
    }

    let res = MAP_TOPIC.with(|p| {p.borrow_mut().remove(&topic_id)});

    if get_variable_type(&res).contains("Topic") {
        Ok(topic_id.to_string())
    } else {
        Err("Could not unregister topic".to_string())
    }   
}

#[ic_cdk_macros::query]
fn topic(topic_id: String) -> Topics {
    let topic = MAP_TOPIC.with(|p| p.borrow().get(&topic_id.clone())).unwrap();

    Topics {
        id: topic_id.clone(),
        name: topic.name,
        description: topic.description,
        namespaces: topic.namespaces,
        active: topic.active,
    }
}

#[ic_cdk_macros::query]
fn topics() -> Vec<Topics> {
    let mut topics: Vec<Topics> = Vec::new();

    MAP_TOPIC.with(|p| {
        for (k, v) in p.borrow().iter() {
            let tpics = Topics {
                id: k,
                name: v.name,
                description: v.description,
                namespaces: v.namespaces,
                active: v.active,
            };

            topics.push(tpics);
        }
    }); 
    topics   
}

#[ic_cdk_macros::query]
fn topic_by_name(topic_name: String) -> Topics {

    let mut topic: Topics = Topics {
        id: "".to_string(), 
        name: "".to_string(), 
        description: "".to_string(), 
        namespaces: Vec::new(), 
        active: true
    };

    MAP_TOPIC.with(|p| {
        for (k, v) in p.borrow().iter() {

            if v.name == topic_name {
                topic = Topics {
                    id: k,
                    name: v.name,
                    description: v.description,
                    namespaces: v.namespaces,
                    active: v.active,
                };
            }
        }
    }); 

    ic_cdk::print(topic.name.to_string());
    //ic_cdk::print(v.name.to_string());

    topic   
}


/******************************************************/
//
//  Subscriber
//
//  subscriber_register         Register subscription
//  subscriber_unregister       Remove subscription
//  subscriber                  Lookup subscriber
//  subscribers                 Get all subscribers
//  subscribers_by_topic        Get all topic subscribers
//  subscribers_by_topic_name   Get all topic subscribers
//
/******************************************************/


#[ic_cdk_macros::update]
pub async fn subscriber_register(subscriber: Subscribers) -> Result<String, String> {

    let mut _id: String = subscriber.id;
    let mut _canister_id: String = subscriber.canister_id;
    let mut _callback: String = subscriber.callback;
    let mut _name: String = subscriber.name;
    let mut _description: String = subscriber.description;
    let mut _topic: String = subscriber.topic;
    let mut _namespace: String = subscriber.namespace;
    let mut _active: bool = subscriber.active;

    let mut id = create_uuid();

    if _id == "".to_string() {
        _id = id.clone();
    }
 
    let result = MAP_SUBSCRIBER.with(|p| p.borrow_mut().insert(_id.clone().to_string(), Subscribers {
        id: _id.clone(),
        canister_id: _canister_id.clone(),
        callback: _callback.clone(),
        name: _name.clone(),
        description: _description.clone(),
        topic: _topic.clone(),
        namespace: _namespace.clone(),
        active: true,
    }));

    if get_variable_type(&result).contains("Subscribers") {
        Ok(_id.clone().to_string())
    } else {
        Err("Could not unregister topic".to_string())
    }   
}

#[ic_cdk_macros::update]
fn subscriber_unregister(subscriber_id: String) -> Result<String, String> {
    let res = MAP_SUBSCRIBER.with(|p| {p.borrow_mut().remove(&subscriber_id)});

    if get_variable_type(&res).contains("Subscribers") {
        Ok(subscriber_id.to_string())
    } else {
        Err("Could not unregister subscriber".to_string())
    }   
}

#[ic_cdk_macros::query]
fn subscriber(subscriber_id: String) -> Subscribers {
    let subscriber = MAP_SUBSCRIBER.with(|p| p.borrow().get(&subscriber_id.clone())).unwrap();

    Subscribers {
        id: subscriber_id.clone(),
        canister_id: subscriber.canister_id,
        callback: subscriber.callback,
        name: subscriber.name,
        description: subscriber.description,
        topic: subscriber.topic,
        namespace: subscriber.namespace,
        active: subscriber.active,
    }
}

#[ic_cdk_macros::query]
fn subscribers() -> Vec<Subscribers> {
    let mut subscribers: Vec<Subscribers> = Vec::new();

    MAP_SUBSCRIBER.with(|p| {
        for (k, v) in p.borrow().iter() {
            let subs = Subscribers {
                id: k,
                name: v.name,
                description: v.description,
                callback: v.callback,
                canister_id: v.canister_id,
                topic: v.topic,
                namespace: v.namespace,
                active: v.active,
            };

            subscribers.push(subs);
        };
    }); 
    subscribers   
}

#[ic_cdk_macros::query]
fn subscribers_by_topic(topic: String) -> Vec<Subscribers> {
    let mut subscribers: Vec<Subscribers> = Vec::new();
    
    MAP_SUBSCRIBER.with(|p| {
        for (k, v) in p.borrow().iter() {

            if v.topic == topic {
                let subs = Subscribers {
                    id: k,
                    name: v.name,
                    description: v.description,
                    callback: v.callback,
                    canister_id: v.canister_id,
                    topic: v.topic,
                    namespace: v.namespace,
                    active: v.active,
                };

                subscribers.push(subs);
            }
        };
    }); 
    subscribers   
}

#[ic_cdk_macros::query]
fn subscribers_by_topic_name(topic_name: String) -> Vec<Subscribers> {
    let mut subscribers: Vec<Subscribers> = Vec::new();

    let topic_id = topic_by_name(topic_name).id; 

    ic_cdk::print(topic_id.clone());
    
    MAP_SUBSCRIBER.with(|p| {
        for (k, v) in p.borrow().iter() {

            ic_cdk::print(v.clone().topic);

            if v.topic == topic_id.clone() {
                let subs = Subscribers {
                    id: k,
                    name: v.name,
                    description: v.description,
                    callback: v.callback,
                    canister_id: v.canister_id,
                    topic: v.topic,
                    namespace: v.namespace,
                    active: v.active,
                };

                subscribers.push(subs);
            }
        };
    }); 
    subscribers   
}



/******************************************************/
//
//  Agent Functions
//
//  agent_subscribe        Register subscription
//  agent_unsubscribe      Remove subscription
//  agent_subscription     Get details of a topic subscription
//  agent_subscriptions    List topic subscriptions
//
/******************************************************/


#[ic_cdk_macros::update]
pub async fn agent_subscribe(topic_name: String, callback: String) -> CallStringResponse {
    let subscriber_principal_id = ic_cdk::caller();

    let topic: Topics = topic_by_name(topic_name);
    let mut min_subs_count = 1000000;
    let mut namespace_id = String::new();

    ic_cdk::print(topic.name.to_string());

    for i in topic.namespaces.iter() {
        ic_cdk::print(i.to_string());

        let ns = namespace(i.to_string()).unwrap();
        let size = ns.subscribers.len();

        if size < min_subs_count {
            min_subs_count = size;
            namespace_id = i.to_string();
        }
    }

    let mut _id = create_uuid();

    let subscriber = Subscribers {
        id: _id.clone().to_string(),
        canister_id: subscriber_principal_id.to_string(),
        callback: callback,
        name: "".to_string(),
        description: "".to_string(),
        topic: topic.id,
        namespace: namespace_id.clone(),
        active: true,
    };

    let sub = subscriber_register(subscriber).await;
    let res = namespace_register_subscriber(namespace_id, _id.clone().to_string());

    CallStringResponse {
        result: _id.clone().to_string()
    }
}

#[ic_cdk_macros::update]
pub async fn agent_unsubscribe(subscription_id: String) -> Result<String, String> {
    let subscriber_principal_id = ic_cdk::caller();
    let mut namespace = namespace_by_subscriber(subscription_id.clone());

    namespace.subscribers.retain(|x| x != &subscription_id.clone());

    let mod_namespace = Namespaces {
        id: namespace.id.clone(),
        name: namespace.name,
        description: namespace.description,
        subscribers: namespace.subscribers,
        active: namespace.active,
    };

    let res_namespace = MAP_NAMESPACE.with(|p| p.borrow_mut().insert(namespace.id.clone(), mod_namespace));

    if get_variable_type(&res_namespace).contains("Namespace") {
        let res_subscriber = MAP_SUBSCRIBER.with(|p| {p.borrow_mut().remove(&subscription_id)});

        if get_variable_type(&res_subscriber).contains("Subscriber") {

            Ok(subscription_id.to_string())
        } else {
            Err("Could not unregister subscription".to_string())
        }   
    } else {
        Err("Could not unregister subscription".to_string())
    }   
}


#[ic_cdk_macros::update]
pub async fn agent_subscription(subscriber_id: String) -> CallSubscriberResponse {
    let subscriber = MAP_SUBSCRIBER.with(|p| p.borrow().get(&subscriber_id)).unwrap();

    CallSubscriberResponse {
        result: Subscriber {
            id: subscriber_id,
            canister_id: subscriber.canister_id,
            callback: subscriber.callback,
            name: subscriber.name,
            description: subscriber.description,
            topic: subscriber.topic,
            namespace: subscriber.namespace,
            active: true,
        }
    }
}

#[ic_cdk_macros::query]
pub async fn agent_subscriptions() -> CallSubscribersResponse { 
    let subscriber_principal_id = ic_cdk::caller();

    let mut subscribers: Vec<Subscribers> = Vec::new();

    MAP_SUBSCRIBER.with(|p| {
        for (k, v) in p.borrow().iter() {

            if v.canister_id == subscriber_principal_id.to_string() {
                let subs = Subscribers {
                    id: k,
                    name: v.name,
                    description: v.description,
                    callback: v.callback,
                    canister_id: v.canister_id,
                    topic: v.topic,
                    namespace: v.namespace,
                    active: v.active,
                };

                subscribers.push(subs);
            }
        };
    }); 

    CallSubscribersResponse {
        result: subscribers
    }
}


/******************************************************/
//
//  CANISTERS
//
//  canister_register               Add canister registry
//  canister_unregister             Remove canister registry
//  canister                        Get a specific canister's details
//  canisters                       Get all canister
//  canisters_remote_set            Set canister settings in 
//                                  remote canisters
//  
/******************************************************/

#[ic_cdk_macros::update]
pub fn canister_register(canister: Canisters) -> Result<String, String> {

    let mut _id = create_uuid();

    let canister_insert = Canisters {
        id: _id.clone().to_string(),
        canister_id: canister.clone().canister_id,
        canister_name: canister.clone().canister_name,
        name: canister.clone().name,
        description: canister.clone().description,
        active: true,
    };

    let result = MAP_CANISTER.with(|p| p.borrow_mut().insert(canister.clone().canister_name.to_string(), canister_insert));

    if get_variable_type(&result).contains("Canisters") {
        Ok(_id.clone().to_string())
    } else {
        Err("Could not register canister".to_string())
    }   

}

#[ic_cdk_macros::update]
pub fn canister_unregister(canister_name: String) -> Result<String, String> {

    let res = MAP_CANISTER.with(|p| {p.borrow_mut().remove(&canister_name)});

    if get_variable_type(&res).contains("Canisters") {
        Ok(format!("Success: Canister {} has been removed", canister_name.to_string()).to_string())
    } else {
        Err("Could not unregister subscriber".to_string())
    }

}

#[ic_cdk_macros::query]
pub fn canister(canister_name: String) -> Option<Canisters> {

    MAP_CANISTER.with(|p| p.borrow().get(&canister_name))
}


#[ic_cdk_macros::query]
pub fn canisters() -> Vec<Canisters> {

    let mut canisters: Vec<Canisters> = Vec::new();

    MAP_CANISTER.with(|p| {
        for (k, v) in p.borrow().iter() {
            let cans = Canisters {
                id: v.id,
                canister_id: v.canister_id,
                canister_name: v.canister_name,
                name: v.name,
                description: v.description,
                active: v.active,
            };

            canisters.push(cans);
        }
    }); 

    canisters
}


#[ic_cdk_macros::update]
pub async fn canisters_remote_set() -> () {
    let canisters = canisters();

    for intercanisters in canisters.iter() {

        if intercanisters.clone().canister_name.to_string() != "registry_backend".to_string() {
            for canister in canisters.iter() {
                let _: () = ic_cdk::call(Principal::from_text(intercanisters.canister_id.to_string()).expect("Could not decode the principal."), "canister_settings_store", (canister.canister_name.to_string(), canister.canister_id.to_string())).await.unwrap();
    
                ic_cdk::print(format!("{} - {}", intercanisters.canister_name.to_string(), canister.canister_id.to_string()));
    
            }
        }       
    }    
}

/*
ic_cdk::spawn( 
    route_message(deserialize_message(msg.clone()))
);
*/