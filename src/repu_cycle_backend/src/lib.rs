use ic_cdk_macros::*;
use ic_cdk::storage;
use std::collections::HashMap;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::api::call::RejectionCode;

// Structure Definitions
#[derive(Clone, Debug, CandidType, Deserialize)]
struct User {
    id: String,
    email: String,
    tokens: u32,
    role: Role,
    preferred_language: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum Role {
    Admin,
    User,
}

#[derive(Clone, CandidType, Deserialize)]
struct UserFootprint {
    waste_generated: u32, 
    recyclable_waste: u32, 
    footprint_score: f32, 
}

#[derive(Clone, CandidType, Deserialize)]
struct Quiz {
    level: u32,
    questions: Vec<String>,
    correct_answers: Vec<String>,
    reward: u32, // Reward tokens for quiz completion
}

#[derive(Clone, CandidType, Deserialize)]
struct Challenge {
    description: String,
    reward_tokens: u32,
}

#[derive(Clone, CandidType, Deserialize)]
struct Token {
    owner: String,
    balance: u32,
}

// Type Aliases
type Users = HashMap<String, User>;
type Footprints = HashMap<String, UserFootprint>;
type Quizzes = HashMap<String, Quiz>;
type Challenges = HashMap<String, Challenge>;
type Ledger = HashMap<String, Token>;
type ActionLog = Vec<String>;

// Initialize all shared storage
#[init]
fn init() {
    storage::stable_save((
        HashMap::<String, User>::new(), 
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(), 
        Vec::<String>::new(),  // Action log for transparency
    )).unwrap();
}

// Logging Function
fn log_action(action: &str) {
    let (_, _, _, _, _, mut log) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    log.push(action.to_string());
    storage::stable_save((
        HashMap::<String, User>::new(), 
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(), 
        log,
    )).unwrap();
}

// User Registration (with roles)
#[update]
fn register_user(id: String, email: String, role: Role, preferred_language: String) -> Result<String, String> {
    let (mut users, _, _, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    
    if users.contains_key(&id) {
        return Err("User already exists".to_string());
    }
    
    users.insert(id.clone(), User { id: id.clone(), email, tokens: 0, role, preferred_language });
    storage::stable_save((
        users,
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(),
        Vec::<String>::new(), // Log initialization
    )).unwrap();

    log_action(&format!("User {} registered", id));
    
    Ok("User registered successfully".to_string())
}

// Get User Information
#[query]
fn get_user(id: String) -> Option<User> {
    let (users, _, _, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    users.get(&id).cloned()
}

// Update User Tokens
#[update]
fn update_user(id: String, tokens: u32) -> Result<String, String> {
    let (mut users, _, _, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    
    match users.get_mut(&id) {
        Some(user) => {
            user.tokens += tokens;
            storage::stable_save((
                users,
                HashMap::<String, UserFootprint>::new(), 
                HashMap::<String, Quiz>::new(), 
                HashMap::<String, Challenge>::new(), 
                HashMap::<String, Token>::new(),
                Vec::<String>::new(),
            )).unwrap();
            log_action(&format!("User {} updated", id));
            Ok("User updated successfully".to_string())
        }
        None => Err("User not found".to_string()),
    }
}

// Admin Feature: Add Quiz
#[update]
fn add_quiz(id: String, level: u32, questions: Vec<String>, correct_answers: Vec<String>, reward: u32) -> Result<String, String> {
    let (users, _, mut quizzes, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    
    match users.get(&id) {
        Some(user) if user.role == Role::Admin => {
            quizzes.insert(id.clone(), Quiz { level, questions, correct_answers, reward });
            storage::stable_save((
                HashMap::<String, User>::new(),
                HashMap::<String, UserFootprint>::new(), 
                quizzes,
                HashMap::<String, Challenge>::new(), 
                HashMap::<String, Token>::new(),
                Vec::<String>::new(),
            )).unwrap();
            log_action(&format!("Admin {} added quiz", id));
            Ok("Quiz added successfully".to_string())
        }
        _ => Err("Unauthorized access".to_string()),
    }
}

// Waste Logging
#[update]
fn log_waste_data(user_id: String, waste_generated: u32, recyclable_waste: u32) -> Result<f32, String> {
    let footprint_score = (waste_generated as f32) * 0.7 - (recyclable_waste as f32) * 0.5;
    let (_, mut footprints, _, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    
    footprints.insert(user_id.clone(), UserFootprint { waste_generated, recyclable_waste, footprint_score });
    storage::stable_save((
        HashMap::<String, User>::new(),
        footprints,
        HashMap::<String, Quiz>::new(),
        HashMap::<String, Challenge>::new(),
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
    )).unwrap();
    
    log_action(&format!("User {} logged waste data", user_id));
    Ok(footprint_score)
}

// Get User Footprint
#[query]
fn get_footprint(user_id: String) -> Option<UserFootprint> {
    let (_, footprints, _, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    footprints.get(&user_id).cloned()
}

// Admin Feature: Add Challenge
#[update]
fn add_challenge(admin_id: String, challenge_id: String, description: String, reward_tokens: u32) -> Result<String, String> {
    let (users, _, _, mut challenges, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    
    match users.get(&admin_id) {
        Some(user) if user.role == Role::Admin => {
            challenges.insert(challenge_id.clone(), Challenge { description, reward_tokens });
            storage::stable_save((
                HashMap::<String, User>::new(),
                HashMap::<String, UserFootprint>::new(),
                HashMap::<String, Quiz>::new(),
                challenges,
                HashMap::<String, Token>::new(),
                Vec::<String>::new(),
            )).unwrap();
            log_action(&format!("Admin {} added challenge", admin_id));
            Ok("Challenge added successfully".to_string())
        }
        _ => Err("Unauthorized access".to_string()),
    }
}

// Reward User for Completing Challenge
#[update]
async fn reward_user(user_id: String, challenge_id: String) -> Result<String, String> {
    let (_, _, _, challenges, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    
    match challenges.get(&challenge_id) {
        Some(challenge) => {
            let result: Result<(), (RejectionCode, String)> = ic_cdk::call(
                ic_cdk::export::Principal::from_text("token_management").unwrap(), 
                "mint_tokens", 
                (user_id.clone(), challenge.reward_tokens)
            ).await;
            
            result.map_err(|(_, e)| e)?;
            log_action(&format!("User {} rewarded for challenge {}", user_id, challenge_id));
            Ok("User rewarded successfully".to_string())
        }
        None => Err("Challenge not found".to_string()),
    }
}

// Token Management: Mint Tokens
#[update]
fn mint_tokens(user_id: String, amount: u32) -> Result<String, String> {
    let (_, _, _, _, mut ledger, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    
    let token = ledger.entry(user_id.clone()).or_insert(Token {
        owner: user_id.clone(),
        balance: 0,
    });
    
    token.balance += amount;
    
    storage::stable_save((
        HashMap::<String, User>::new(),
        HashMap::<String, UserFootprint>::new(),
        HashMap::<String, Quiz>::new(),
        HashMap::<String, Challenge>::new(),
        ledger,
        Vec::<String>::new(),
    )).unwrap();
    
    log_action(&format!("Minted {} tokens to {}", amount, user_id));
    Ok(format!("{} tokens minted to {}", amount, user_id))
}

// Token Management: Burn Tokens
#[update]
fn burn_tokens(user_id: String, amount: u32) -> Result<String, String> {
    let (_, _, _, _, mut ledger, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    
    match ledger.get_mut(&user_id) {
        Some(token) => {
            if token.balance >= amount {
                token.balance -= amount;
                storage::stable_save((
                    HashMap::<String, User>::new(),
                    HashMap::<String, UserFootprint>::new(),
                    HashMap::<String, Quiz>::new(),
                    HashMap::<String, Challenge>::new(),
                    ledger,
                    Vec::<String>::new(),
                )).unwrap();
                log_action(&format!("Burned {} tokens from {}", amount, user_id));
                Ok(format!("{} tokens burned from {}", amount, user_id))
            } else {
                Err("Insufficient balance".to_string())
            }
        }
        None => Err("User not found in ledger".to_string()),
    }
}

// Admin Feature: Get Logs for Transparency
#[query]
fn get_action_logs() -> ActionLog {
    let (_, _, _, _, _, logs) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog)>().unwrap();
    logs
}
