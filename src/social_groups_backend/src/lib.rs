#[macro_use]
extern crate serde;

use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = RefCell<u64>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, BoundedStorable)]
struct DeveloperProfile {
    name: String,
    location: String,
    ideas: Vec<String>,
    groups: Vec<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, BoundedStorable)]
struct SocialGroup {
    id: u64,
    name: String,
    idea: String,
    members: Vec<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, BoundedStorable)]
struct Message {
    sender_id: u64,
    group_id: u64,
    content: String,
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
    static DEVELOPER_ID_COUNTER: IdCell = RefCell::new(0);
    static GROUP_ID_COUNTER: IdCell = RefCell::new(0);
    static MESSAGE_ID_COUNTER: IdCell = RefCell::new(0);
    static DEVELOPER_PROFILE_STORAGE: RefCell<StableBTreeMap<u64, DeveloperProfile, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))))
    );
    static SOCIAL_GROUP_STORAGE: RefCell<StableBTreeMap<u64, SocialGroup, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))))
    );
    static MESSAGES_STORAGE: RefCell<StableBTreeMap<u64, Message, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))))
    );
}

#[ic_cdk::update]
fn create_developer_profile(name: String, location: String, ideas: Vec<String>) -> Result<DeveloperProfile, String> {
    let id = DEVELOPER_ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() += 1;
        current_value
    });

    let developer_profile = DeveloperProfile {
        name,
        location,
        ideas,
        groups: Vec::new(),
    };
    do_insert_developer_profile(id, &developer_profile);
    Ok(developer_profile)
}

fn do_insert_developer_profile(id: u64, developer_profile: &DeveloperProfile) {
    DEVELOPER_PROFILE_STORAGE.with(|service| {
        service
            .borrow_mut()
            .insert(id, developer_profile.clone());
    });
}

#[ic_cdk::query]
fn get_developer_profile(id: u64) -> Result<DeveloperProfile, String> {
    DEVELOPER_PROFILE_STORAGE.with(|service| {
        service
            .borrow()
            .get(&id)
            .ok_or_else(|| format!("Developer with id={} not found", id))
    })
}

// Other functions (get_all_developer_profiles, create_social_group, etc.) remain unchanged.


#[ic_cdk::query]
fn get_all_developer_profiles() -> Vec<DeveloperProfile> {
    let developer_mapping: Vec<(u64, DeveloperProfile)> =
        DEVELOPER_PROFILE_STORAGE.with(|service| service.borrow().iter().collect());
    developer_mapping
        .into_iter()
        .map(|(_, developer)| developer)
        .collect()
}

#[ic_cdk::update]
fn create_social_group(name: String, idea: String) -> Result<SocialGroup, String> {
    let id = GROUP_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let social_group = SocialGroup {
        id,
        name,
        idea,
        members: Vec::new(),
    };
    do_insert_social_group(id, &social_group);
    Ok(social_group)
}

fn do_insert_social_group(id: u64, social_group: &SocialGroup) {
    SOCIAL_GROUP_STORAGE.with(|service| {
        service
            .borrow_mut()
            .insert(id, social_group.clone())
    });
}

#[ic_cdk::query]
fn get_social_group(id: u64) -> Result<SocialGroup, String> {
    SOCIAL_GROUP_STORAGE.with(|service| {
        service
            .borrow_mut()
            .get(&id)
            .ok_or(format!("Social group with id={} not found", id))
    })
}

#[ic_cdk::query]
fn get_all_social_groups() -> Vec<SocialGroup> {
    let social_group_mapping: Vec<(u64, SocialGroup)> =
        SOCIAL_GROUP_STORAGE.with(|service| service.borrow().iter().collect());
    social_group_mapping
        .into_iter()
        .map(|(_, social_group)| social_group)
        .collect()
}

#[ic_cdk::update]
fn join_social_group(developer_id: u64, group_id: u64) -> Result<(), String> {
    let developer_profile = get_developer_profile(developer_id)?;
    let social_group = get_social_group(group_id)?;

    if !developer_profile.ideas.contains(&social_group.idea) {
        return Err(format!(
            "Developer {} cannot join group {} because their ideas do not match",
            developer_id, group_id
        ));
    }

    SOCIAL_GROUP_STORAGE.with(|service| {
        let mut group = service
            .borrow_mut()
            .get_mut(&group_id)
            .ok_or(format!("Social group with id={} not found", group_id))?;

        group.members.push(developer_id);
    });

    DEVELOPER_PROFILE_STORAGE.with(|service| {
        let mut developer = service
            .borrow_mut()
            .get_mut(&developer_id)
            .ok_or(format!("Developer with id={} not found", developer_id))?;

        developer.groups.push(group_id);
    });

    Ok(())
}

#[ic_cdk::update]
fn send_message(sender_id: u64, group_id: u64, content: String) -> Result<Message, String> {
    let developer_profile = get_developer_profile(sender_id)?;
    let social_group = get_social_group(group_id)?;

    if !social_group.members.contains(&sender_id) {
        return Err(format!(
            "Developer {} is not a member of group {} and cannot send messages to it",
            sender_id, group_id
        ));
    }

    let id = MESSAGE_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let message = Message {
        sender_id,
        group_id,
        content,
    };
    do_insert_message(id, &message);
    Ok(message)
}

fn do_insert_message(id: u64, message: &Message) {
    MESSAGES_STORAGE.with(|service| {
        service
            .borrow_mut()
            .insert(id, message.clone())
    });
}

#[ic_cdk::query]
fn get_message(id: u64) -> Result<Message, String> {
    MESSAGES_STORAGE.with(|service| {
        service
            .borrow_mut()
            .get(&id)
            .ok_or(format!("Message with id={} not found", id))
    })
}

#[ic_cdk::query]
fn get_all_messages() -> Vec<Message> {
    let message_mapping: Vec<(u64, Message)> =
        MESSAGES_STORAGE.with(|service| service.borrow().iter().collect());
    message_mapping
        .into_iter()
        .map(|(_, message)| message)
        .collect()
}

