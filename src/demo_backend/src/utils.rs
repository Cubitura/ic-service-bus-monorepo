use uuid_by_string::generate_uuid::{generate_uuid};


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

