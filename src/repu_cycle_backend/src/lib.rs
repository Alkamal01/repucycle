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
}

#[derive(Clone, CandidType, Deserialize)]
struct UserFootprint {
    waste_generated: u32, // kg of waste generated
    recyclable_waste: u32, // kg of recyclable waste
    footprint_score: f32, // Calculated carbon footprint score
}

#[derive(Clone, CandidType, Deserialize)]
struct Quiz {
    questions: Vec<String>,
    correct_answers: Vec<String>,
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

// Initialize all shared storage
#[init]
fn init() {
    storage::stable_save((
        HashMap::<String, User>::new(), // Explicit type annotation
        HashMap::<String, UserFootprint>::new(), // Explicit type annotation
        HashMap::<String, Quiz>::new(), // Explicit type annotation
        HashMap::<String, Challenge>::new(), // Explicit type annotation
        HashMap::<String, Token>::new() // Explicit type annotation
    )).unwrap();
}

// User Registration
#[update]
fn register_user(id: String, email: String) -> Result<String, String> {
    let (mut users, _, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    
    if users.contains_key(&id) {
        return Err("User already exists".to_string());
    }
    
    users.insert(id.clone(), User { id: id.clone(), email, tokens: 0 });
    storage::stable_save((
        users,
        HashMap::<String, UserFootprint>::new(), // Updated with actual types
        HashMap::<String, Quiz>::new(), // Updated with actual types
        HashMap::<String, Challenge>::new(), // Updated with actual types
        HashMap::<String, Token>::new(), // Updated with actual types
    )).unwrap();
    
    Ok("User registered successfully".to_string())
}

#[query]
fn get_user(id: String) -> Option<User> {
    let (users, _, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    users.get(&id).cloned()
}

#[update]
fn update_user(id: String, tokens: u32) -> Result<String, String> {
    let (mut users, _, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    
    match users.get_mut(&id) {
        Some(user) => {
            user.tokens += tokens;
            storage::stable_save((
                users,
                HashMap::<String, UserFootprint>::new(), // Updated with actual types
                HashMap::<String, Quiz>::new(), // Updated with actual types
                HashMap::<String, Challenge>::new(), // Updated with actual types
                HashMap::<String, Token>::new(), // Updated with actual types
            )).unwrap();
            Ok("User updated successfully".to_string())
        }
        None => Err("User not found".to_string()),
    }
}

// Waste Logging
#[update]
fn log_waste_data(user_id: String, waste_generated: u32, recyclable_waste: u32) -> Result<f32, String> {
    let footprint_score = (waste_generated as f32) * 0.7 - (recyclable_waste as f32) * 0.5;
    let (_, mut footprints, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    
    footprints.insert(user_id.clone(), UserFootprint { waste_generated, recyclable_waste, footprint_score });
    storage::stable_save((
        HashMap::<String, User>::new(), // Updated with actual types
        footprints,
        HashMap::<String, Quiz>::new(), // Updated with actual types
        HashMap::<String, Challenge>::new(), // Updated with actual types
        HashMap::<String, Token>::new(), // Updated with actual types
    )).unwrap();
    
    Ok(footprint_score)
}

#[query]
fn get_footprint(user_id: String) -> Option<UserFootprint> {
    let (_, footprints, _, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    footprints.get(&user_id).cloned()
}

// Quiz Management
#[update]
fn add_quiz(id: String, questions: Vec<String>, correct_answers: Vec<String>) -> Result<String, String> {
    let (_, _, mut quizzes, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    
    quizzes.insert(id.clone(), Quiz { questions, correct_answers });
    storage::stable_save((
        HashMap::<String, User>::new(), // Updated with actual types
        HashMap::<String, UserFootprint>::new(), // Updated with actual types
        quizzes,
        HashMap::<String, Challenge>::new(), // Updated with actual types
        HashMap::<String, Token>::new(), // Updated with actual types
    )).unwrap();
    
    Ok("Quiz added successfully".to_string())
}

#[update]
fn submit_quiz(id: String, user_answers: Vec<String>) -> Result<u32, String> {
    let (_, _, quizzes, _, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    
    match quizzes.get(&id) {
        Some(quiz) => {
            let correct_answers = &quiz.correct_answers;
            let score = user_answers.iter()
                .enumerate()
                .filter(|(i, user_answer)| *user_answer == &correct_answers[*i]) // Use reference here
                .count() as u32;
            Ok(score)
        }
        None => Err("Quiz not found".to_string()),
    }
}


// Challenge Management
#[update]
fn add_challenge(id: String, description: String, reward_tokens: u32) -> Result<String, String> {
    let (_, _, _, mut challenges, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    
    challenges.insert(id.clone(), Challenge { description, reward_tokens });
    storage::stable_save((
        HashMap::<String, User>::new(), // Updated with actual types
        HashMap::<String, UserFootprint>::new(), // Updated with actual types
        HashMap::<String, Quiz>::new(), // Updated with actual types
        challenges,
        HashMap::<String, Token>::new(), // Updated with actual types
    )).unwrap();
    
    Ok("Challenge added successfully".to_string())
}

#[update]
async fn reward_user(user_id: String, challenge_id: String) -> Result<String, String> {
    let (_, _, _, challenges, _) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    
    match challenges.get(&challenge_id) {
        Some(challenge) => {
            // Use the `?` operator to propagate the error
            let result: Result<(), (RejectionCode, String)> = ic_cdk::call(
                ic_cdk::export::Principal::from_text("token_management").unwrap(), 
                "mint_tokens", 
                (user_id.clone(), challenge.reward_tokens)
            ).await;
            
            result.map_err(|(_, e)| e)?;
            
            Ok("User rewarded successfully".to_string())
        }
        None => Err("Challenge not found".to_string()),
    }
}

// Token Management
#[update]
fn mint_tokens(user_id: String, amount: u32) -> Result<String, String> {
    let (_, _, _, _, mut ledger) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    
    let token = ledger.entry(user_id.clone()).or_insert(Token {
        owner: user_id.clone(),
        balance: 0,
    });
    token.balance += amount;
    
    storage::stable_save((
        HashMap::<String, User>::new(), // Updated with actual types
        HashMap::<String, UserFootprint>::new(), // Updated with actual types
        HashMap::<String, Quiz>::new(), // Updated with actual types
        HashMap::<String, Challenge>::new(), // Updated with actual types
        ledger
    )).unwrap();
    Ok("Tokens minted successfully".to_string())
}

#[update]
fn transfer_tokens(from: String, to: String, amount: u32) -> Result<String, String> {
    let (_, _, _, _, mut ledger) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    
    // First, try to get the mutable reference for the sender
    if let Some(sender_token) = ledger.get_mut(&from) {
        // Check if the sender has sufficient balance
        if sender_token.balance >= amount {
            // Reduce the sender's balance
            sender_token.balance -= amount;

            // Safely handle the receiver
            let receiver = ledger.entry(to.clone()).or_insert(Token { owner: to.clone(), balance: 0 });
            receiver.balance += amount; // Increase the receiver's balance

            // Save the updated ledger back to stable storage
            storage::stable_save((
                HashMap::<String, User>::new(), // Updated with actual types
                HashMap::<String, UserFootprint>::new(), // Updated with actual types
                HashMap::<String, Quiz>::new(), // Updated with actual types
                HashMap::<String, Challenge>::new(), // Updated with actual types
                ledger
            )).unwrap();

            Ok("Tokens transferred successfully".to_string())
        } else {
            Err("Insufficient balance".to_string())
        }
    } else {
        Err("Sender not found".to_string())
    }
}


#[query]
fn get_balance(user_id: String) -> Result<u32, String> {
    let (_, _, _, _, ledger) = storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger)>().unwrap();
    
    match ledger.get(&user_id) {
        Some(token) => Ok(token.balance),
        None => Err("User not found".to_string()),
    }
}
