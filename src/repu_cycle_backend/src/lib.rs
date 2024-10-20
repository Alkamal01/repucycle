use ic_cdk_macros::*;
use ic_cdk::storage;
use std::collections::HashMap;
use ic_cdk::export::candid::{CandidType, Deserialize};
use sha2::{Sha256, Digest}; // For password hashing
use uuid::Uuid; // For generating unique session tokens
use serde_json::json; // For structured logging
use chrono::{Utc, Duration}; // For managing expiration times

// Custom Error Type
#[derive(Debug, CandidType, Deserialize)]
enum AppError {
    UserAlreadyExists,
    InvalidCredentials,
    UserNotFound,
    SessionTokenExpired,
    ChallengeNotFound,
    CourseAlreadyExists,
    QuizAlreadyExists,
    RequiredCoursesNotCompleted,
    FeedbackError,
    StorageError(String),
    NotificationError,
    InvalidReward,
}

// Implementing Display for AppError for easier debugging
impl ToString for AppError {
    fn to_string(&self) -> String {
        match self {
            AppError::UserAlreadyExists => "User already exists".to_string(),
            AppError::InvalidCredentials => "Invalid credentials".to_string(),
            AppError::UserNotFound => "User not found".to_string(),
            AppError::SessionTokenExpired => "Session token expired".to_string(),
            AppError::ChallengeNotFound => "Challenge not found".to_string(),
            AppError::CourseAlreadyExists => "Course already exists".to_string(),
            AppError::QuizAlreadyExists => "Quiz already exists".to_string(),
            AppError::RequiredCoursesNotCompleted => "You have not completed the required courses or quizzes".to_string(),
            AppError::FeedbackError => "Error collecting feedback".to_string(),
            AppError::StorageError(e) => format!("Storage error: {}", e),
            AppError::NotificationError => "Error sending notification".to_string(),
            AppError::InvalidReward => "Invalid reward request".to_string(),
        }
    }
}

// Structure Definitions
#[derive(Clone, Debug, CandidType, Deserialize)]
struct User {
    id: String,
    full_name: String,
    email: String,
    hashed_password: String,
    tokens: u32,
    role: Role,
    preferred_language: String,
    session_token: Option<SessionToken>,
    achievements: Vec<String>,
    challenges_completed: Vec<String>,
    completed_courses: Vec<String>,
    passed_quizzes: Vec<String>,
    notifications: Vec<String>, // For social notifications
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
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
    options: Vec<Vec<String>>,
    correct_answers: Vec<String>,
    reward: u32,
}

#[derive(Clone, CandidType, Deserialize)]
struct Challenge {
    description: String,
    reward_tokens: u32,
    required_courses: Vec<String>,
    required_quizzes: Vec<String>,
    participants: Vec<String>,
}

#[derive(Clone, CandidType, Deserialize)]
struct Token {
    owner: String,
    balance: u32,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct SessionToken {
    token: String,
    expires_at: i64,
}

#[derive(Clone, CandidType, Deserialize)]
struct Course {
    title: String,
    levels: HashMap<u32, Quiz>,
    educational_resources: Vec<String>, // Educational resources
}

#[derive(Clone, CandidType, Deserialize)]
struct Notification {
    user_id: String,
    message: String,
    timestamp: i64,
    notification_type: NotificationType,
}

#[derive(Clone, CandidType, Deserialize)]
enum NotificationType {
    CourseAdded,
    ChallengeAdded,
    ChallengeStarted,
    ChallengeParticipated,
    ChallengeReminder,
    AchievementShared, // New notification type for achievements
}

#[derive(Clone, CandidType, Deserialize)]
struct Feedback {
    user_id: String,
    feedback: String,
}

#[derive(Clone, CandidType, Deserialize)]
struct Reward {
    id: String,
    description: String,
    cost_tokens: u32,
}

type Users = HashMap<String, User>;
type Footprints = HashMap<String, UserFootprint>;
type Quizzes = HashMap<String, Quiz>;
type Challenges = HashMap<String, Challenge>;
type Ledger = HashMap<String, Token>;
type ActionLog = Vec<String>;
type Courses = HashMap<String, Course>;
type Notifications = HashMap<String, Vec<Notification>>;
type Feedbacks = Vec<Feedback>;
type Rewards = HashMap<String, Reward>;

#[init]
fn init() {
    storage::stable_save((
        HashMap::<String, User>::new(),
        HashMap::<String, UserFootprint>::new(),
        HashMap::<String, Quiz>::new(),
        HashMap::<String, Challenge>::new(),
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        HashMap::<String, Course>::new(),
        HashMap::<String, Vec<Notification>>::new(),
        Vec::<Feedback>::new(),
        HashMap::<String, Reward>::new(),
    )).unwrap();
}

fn restore_storage() -> (Users, Footprints, Quizzes, Challenges, Ledger, ActionLog, Courses, Notifications, Feedbacks, Rewards) {
    storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog, Courses, Notifications, Feedbacks, Rewards)>().unwrap()
}

fn save_storage(
    users: Users,
    footprints: Footprints,
    quizzes: Quizzes,
    challenges: Challenges,
    tokens: Ledger,
    log: ActionLog,
    courses: Courses,
    notifications: Notifications,
    feedbacks: Feedbacks,
    rewards: Rewards, // Include rewards in storage
) -> Result<(), AppError> {
    storage::stable_save((
        users,
        footprints,
        quizzes,
        challenges,
        tokens,
        log,
        courses,
        notifications,
        feedbacks,
        rewards,
    )).map_err(|e| AppError::StorageError(e.to_string()))
}

fn log_action(action: &str) -> Result<(), AppError> {
    let (_, _, _, _, _, mut log, _, _, _, _) = restore_storage();
    log.push(json!({ "action": action, "timestamp": ic_cdk::api::time() as i64 }).to_string()); // Convert timestamp
    save_storage(
        HashMap::<String, User>::new(),
        HashMap::<String, UserFootprint>::new(),
        HashMap::<String, Quiz>::new(),
        HashMap::<String, Challenge>::new(),
        HashMap::<String, Token>::new(),
        log,
        HashMap::<String, Course>::new(),
        HashMap::<String, Vec<Notification>>::new(),
        Vec::<Feedback>::new(),
        HashMap::<String, Reward>::new(), // Include rewards in storage
    )?;
    Ok(())
}

// Hash Password with Salt
fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.update(salt);
    format!("{:x}", hasher.finalize())
}

// User Registration
#[update]
fn register_user(id: String, full_name: String, email: String, password: String, role: Option<Role>, preferred_language: String) -> Result<String, AppError> {
    let (mut users, _, _, _, _, _, _, _, _, _) = restore_storage();

    if users.contains_key(&id) {
        return Err(AppError::UserAlreadyExists);
    }

    let salt = Uuid::new_v4().to_string(); // Generate salt
    let hashed_password = hash_password(&password, &salt);
    let user_role = role.unwrap_or(Role::User);

    users.insert(id.clone(), User { 
        id: id.clone(), 
        full_name, 
        email, 
        hashed_password, 
        tokens: 0, 
        role: user_role, 
        preferred_language,
        session_token: None,
        achievements: Vec::new(), 
        challenges_completed: Vec::new(), 
        completed_courses: Vec::new(), 
        passed_quizzes: Vec::new(),
        notifications: Vec::new(), // Initialize notifications
    });

    save_storage(
        users,
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        HashMap::<String, Course>::new(), 
        HashMap::<String, Vec<Notification>>::new(), 
        Vec::<Feedback>::new(),
        HashMap::<String, Reward>::new(),
    )?;
    
    log_action(&format!("User {} registered", id))?;
    
    Ok("User registered successfully".to_string())
}

// Authenticate User
#[update]
fn login_user(id: String, password: String) -> Result<String, AppError> {
    let (mut users, _, _, _, _, _, _, _, _, _) = restore_storage();

    match users.get_mut(&id) {
        Some(user) if user.hashed_password == hash_password(&password, "") => {
            let session_token = SessionToken {
                token: Uuid::new_v4().to_string(), // Secure token generation
                expires_at: (Utc::now() + Duration::hours(1)).timestamp(),
            };
            user.session_token = Some(session_token.clone());
            save_storage(
                users,
                HashMap::<String, UserFootprint>::new(),
                HashMap::<String, Quiz>::new(),
                HashMap::<String, Challenge>::new(),
                HashMap::<String, Token>::new(),
                Vec::<String>::new(),
                HashMap::<String, Course>::new(),
                HashMap::<String, Vec<Notification>>::new(),
                Vec::<Feedback>::new(),
                HashMap::<String, Reward>::new(),
            )?;
            Ok(session_token.token)
        }
        Some(_) => Err(AppError::InvalidCredentials),
        None => Err(AppError::UserNotFound),
    }
}

// Check Session Token
fn check_session_token(user: &User) -> Result<(), AppError> {
    if let Some(token) = &user.session_token {
        if token.expires_at < ic_cdk::api::time() as i64 { // Convert timestamp
            return Err(AppError::SessionTokenExpired);
        }
        Ok(())
    } else {
        Err(AppError::SessionTokenExpired)
    }
}

// Add a Course
#[update]
fn add_course(title: String, levels: HashMap<u32, Quiz>, educational_resources: Vec<String>) -> Result<String, AppError> {
    let (mut users, _, _, _, _, _, mut courses, _, _, _) = restore_storage();
    
    if courses.contains_key(&title) {
        return Err(AppError::CourseAlreadyExists);
    }

    courses.insert(title.clone(), Course { 
        title: title.clone(), 
        levels, 
        educational_resources 
    });

    save_storage(
        users,
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        courses,
        HashMap::<String, Vec<Notification>>::new(), 
        Vec::<Feedback>::new(),
        HashMap::<String, Reward>::new(),
    )?;
    
    log_action(&format!("Course {} added", title))?;
    
    Ok("Course added successfully".to_string())
}

// Submit Feedback
#[update]
fn submit_feedback(user_id: String, feedback: String) -> Result<String, AppError> {
    let (mut users, _, _, _, _, _, _, _, mut feedbacks, _) = restore_storage();

    if !users.contains_key(&user_id) {
        return Err(AppError::UserNotFound);
    }

    feedbacks.push(Feedback { user_id: user_id.clone(), feedback });
    
    save_storage(
        users,
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        HashMap::<String, Course>::new(), 
        HashMap::<String, Vec<Notification>>::new(), 
        feedbacks,
        HashMap::<String, Reward>::new(),
    )?;
    
    log_action(&format!("Feedback submitted by user {}", user_id))?;
    
    Ok("Feedback submitted successfully".to_string())
}

// Add a Challenge
#[update]
fn add_challenge(description: String, reward_tokens: u32, required_courses: Vec<String>, required_quizzes: Vec<String>) -> Result<String, AppError> {
    let (mut users, _, _, mut challenges, _, _, _, _, _, _) = restore_storage();

    let id = Uuid::new_v4().to_string();
    challenges.insert(id.clone(), Challenge { 
        description, 
        reward_tokens, 
        required_courses, 
        required_quizzes,
        participants: Vec::new(), 
    });

    save_storage(
        users,
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        challenges,
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        HashMap::<String, Course>::new(), 
        HashMap::<String, Vec<Notification>>::new(), 
        Vec::<Feedback>::new(),
        HashMap::<String, Reward>::new(),
    )?;
    
    log_action(&format!("Challenge {} added", id))?;
    
    Ok("Challenge added successfully".to_string())
}

// Submit a Social Notification
#[update]
fn send_notification(from_user: String, to_user: String, message: String) -> Result<String, AppError> {
    let (mut users, _, _, _, _, _, _, mut notifications, _, _) = restore_storage();

    if !users.contains_key(&from_user) || !users.contains_key(&to_user) {
        return Err(AppError::UserNotFound);
    }

    let notification = Notification {
        user_id: to_user.clone(),
        message: format!("{}: {}", from_user, message),
        timestamp: ic_cdk::api::time() as i64,
        notification_type: NotificationType::AchievementShared,
    };

    notifications.entry(to_user.clone()).or_insert(Vec::new()).push(notification);
    users.get_mut(&from_user).unwrap().notifications.push(format!("You sent a message to {}", to_user));

    save_storage(users, HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new(), Vec::new(), HashMap::new(), notifications, Vec::new(), HashMap::new())?;

    log_action(&format!("Notification sent from {} to {}", from_user, to_user))?;
    
    Ok("Notification sent successfully".to_string())
}

// Redeem Rewards
#[update]
fn redeem_reward(user_id: String, reward_id: String) -> Result<String, AppError> {
    let (mut users, _, _, _, _, _, _, _, _, mut rewards) = restore_storage();

    if let Some(user) = users.get_mut(&user_id) {
        if let Some(reward) = rewards.get(&reward_id) {
            if user.tokens >= reward.cost_tokens {
                user.tokens -= reward.cost_tokens;
                // Logic for granting the reward can go here
                log_action(&format!("User {} redeemed reward {}", user_id, reward_id))?;
                return Ok(format!("Reward {} redeemed successfully!", reward_id));
            } else {
                return Err(AppError::InvalidReward);
            }
        } else {
            return Err(AppError::InvalidReward);
        }
    }

    Err(AppError::UserNotFound)
}

// Leaderboard Retrieval
#[query]
fn get_leaderboard() -> Vec<(String, u32)> {
    let (users, _, _, _, _, _, _, _, _, _) = restore_storage();
    let mut leaderboard: Vec<(String, u32)> = users.iter()
        .map(|(id, user)| (id.clone(), user.tokens))
        .collect();
    
    leaderboard.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by tokens descending
    leaderboard
}

// Utility to validate data
fn validate_data() -> Result<(), AppError> {
    let (_, _footprints, _quizzes, _challenges, _, _, courses, _, _, _) = restore_storage();
    
    // Perform data validation here...
    
    Ok(())
}

// Notification Management
#[update]
fn add_notification(user_id: String, message: String, notification_type: NotificationType) -> Result<String, AppError> {
    let (mut users, _, _, _, _, _, _, mut notifications, _, _) = restore_storage();

    if !users.contains_key(&user_id) {
        return Err(AppError::UserNotFound);
    }

    let notification = Notification {
        user_id: user_id.clone(),
        message,
        timestamp: ic_cdk::api::time() as i64,
        notification_type,
    };

    notifications.entry(user_id.clone()).or_insert(Vec::new()).push(notification);

    save_storage(users, HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new(), Vec::new(), HashMap::new(), notifications, Vec::new(), HashMap::new())?;

    log_action(&format!("Notification added for user {}", user_id))?;

    Ok("Notification added successfully".to_string())
}