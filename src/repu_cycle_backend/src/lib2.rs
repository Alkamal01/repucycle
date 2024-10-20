use ic_cdk_macros::*;
use ic_cdk::storage;
use std::collections::HashMap;
use ic_cdk::export::candid::{CandidType, Deserialize};
use sha2::{Sha256, Digest}; // For password hashing
use serde_json::json; // For structured logging
use chrono::{Utc, Duration}; // For managing expiration times

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
    session_token: Option<SessionToken>, // For user session management
    achievements: Vec<String>, // For storing achievements
    challenges_completed: Vec<String>, // Store completed challenges
    completed_courses: Vec<String>, // Store completed courses
    passed_quizzes: Vec<String>, // Store passed quizzes
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
    required_courses: Vec<String>, // List of required courses
    required_quizzes: Vec<String>,  // List of required quizzes
    participants: Vec<String>, // Track participants
}

#[derive(Clone, CandidType, Deserialize)]
struct Token {
    owner: String,
    balance: u32,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct SessionToken {
    token: String,
    expires_at: i64, // Expiration timestamp
}

#[derive(Clone, CandidType, Deserialize)]
struct Course {
    title: String,
    levels: HashMap<u32, Quiz>, // Mapping of level number to Quiz
    educational_resources: Vec<String>, // Educational content
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
}

#[derive(Clone, CandidType, Deserialize)]
struct Feedback {
    user_id: String,
    feedback: String,
}

// Type Aliases
type Users = HashMap<String, User>;
type Footprints = HashMap<String, UserFootprint>;
type Quizzes = HashMap<String, Quiz>;
type Challenges = HashMap<String, Challenge>;
type Ledger = HashMap<String, Token>;
type ActionLog = Vec<String>;
type Courses = HashMap<String, Course>; // Keyed by course title
type Notifications = HashMap<String, Vec<Notification>>; // Notifications for each user
type Feedbacks = Vec<Feedback>; // Store user feedback

// Initialize all shared storage
#[init]
fn init() {
    storage::stable_save((
        HashMap::<String, User>::new(), 
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(), 
        Vec::<String>::new(),  
        HashMap::<String, Course>::new(), // Initialize courses
        HashMap::<String, Vec<Notification>>::new(), // Notifications
        Vec::<Feedback>::new(), // User feedback
    )).unwrap();
}

// Helper function to handle stable storage operations
fn restore_storage() -> (Users, Footprints, Quizzes, Challenges, Ledger, ActionLog, Courses, Notifications, Feedbacks) {
    storage::stable_restore::<(Users, Footprints, Quizzes, Challenges, Ledger, ActionLog, Courses, Notifications, Feedbacks)>().unwrap()
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
) -> Result<(), String> {
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
    )).map_err(|e| format!("Failed to save storage: {}", e)) // Convert the error into a String
}

// Logging Function
fn log_action(action: &str) {
    let (_, _, _, _, _, mut log, _, _, _) = restore_storage();
    log.push(json!({ "action": action, "timestamp": ic_cdk::api::time() }).to_string());
    save_storage(
        HashMap::<String, User>::new(),
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(),
        log,
        HashMap::<String, Course>::new(), // Empty courses for log action
        HashMap::<String, Vec<Notification>>::new(), // Empty notifications for log action
        Vec::<Feedback>::new(), // Empty feedback for log action
    ).unwrap(); // Error handling
}

// Hash Password
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    format!("{:x}", hasher.finalize())
}

// User Registration (with roles)
#[update]
fn register_user(id: String, full_name: String, email: String, password: String, role: Option<Role>, preferred_language: String) -> Result<String, String> {
    let (mut users, _, _, _, _, _, _, _, _) = restore_storage();

    if users.contains_key(&id) {
        return Err("User already exists".to_string());
    }

    let hashed_password = hash_password(&password);
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
        achievements: Vec::new(), // Initialize empty achievements
        challenges_completed: Vec::new(), // Initialize empty challenges
        completed_courses: Vec::new(), // Initialize empty completed courses
        passed_quizzes: Vec::new(), // Initialize empty passed quizzes
    });

    save_storage(
        users,
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        HashMap::<String, Course>::new(), // Empty courses
        HashMap::<String, Vec<Notification>>::new(), // Empty notifications
        Vec::<Feedback>::new(), // Empty feedback
    )?; // Now this works

    log_action(&format!("User {} registered", id));
    
    Ok("User registered successfully".to_string())
}

// Authenticate User
#[update]
fn login_user(id: String, password: String) -> Result<String, String> {
    let (mut users, _, _, _, _, _, _, _, _) = restore_storage();

    match users.get_mut(&id) {
        Some(user) if user.hashed_password == hash_password(&password) => {
            let session_token = SessionToken {
                token: format!("token_{}", id), // Implement a more secure token generation
                expires_at: (Utc::now() + Duration::hours(1)).timestamp(), // Set token expiration to 1 hour from now
            };
            user.session_token = Some(session_token.clone());
            save_storage(
                users,
                HashMap::<String, UserFootprint>::new(), 
                HashMap::<String, Quiz>::new(), 
                HashMap::<String, Challenge>::new(), 
                HashMap::<String, Token>::new(),
                Vec::<String>::new(),
                HashMap::<String, Course>::new(), // Empty courses
                HashMap::<String, Vec<Notification>>::new(), // Empty notifications
                Vec::<Feedback>::new(), // Empty feedback
            ).unwrap(); // Error handling
            log_action(&format!("User {} logged in", id));
            Ok(session_token.token)
        }
        _ => Err("Invalid credentials".to_string()),
    }
}

// Check if user is logged in
fn is_logged_in(user_id: &str) -> Result<User, String> { 
    let (users, _, _, _, _, _, _, _, _) = restore_storage();

    users.get(user_id).ok_or("User not found".to_string())
        .and_then(|user| {
            if let Some(token) = &user.session_token {
                if token.expires_at > Utc::now().timestamp() {
                    Ok(user.clone()) // Return a cloned User instead of a reference
                } else {
                    Err("Session token expired".to_string())
                }
            } else {
                Err("User not logged in".to_string())
            }
        })
}

// User Actions Requiring Login
#[update]
fn update_user(id: String, tokens: u32) -> Result<String, String> {
    let user = is_logged_in(&id)?;
    
    // Update user tokens
    let (mut users, _, _, _, _, _, _, _, _) = restore_storage();
    if let Some(user) = users.get_mut(&id) {
        user.tokens += tokens;
        save_storage(
            users,
            HashMap::<String, UserFootprint>::new(), 
            HashMap::<String, Quiz>::new(), 
            HashMap::<String, Challenge>::new(), 
            HashMap::<String, Token>::new(),
            Vec::<String>::new(),
            HashMap::<String, Course>::new(), // Empty courses
            HashMap::<String, Vec<Notification>>::new(), // Empty notifications
            Vec::<Feedback>::new(), // Empty feedback
        )?;
        log_action(&format!("User {} updated tokens by {}", id, tokens));
        return Ok("Tokens updated".to_string());
    }
    Err("User not found".to_string())
}

// Challenge Participation
#[update]
fn participate_in_challenge(user_id: String, challenge_id: String) -> Result<String, String> {
    let (mut users, mut challenges, _, _, _, _, _, mut notifications, _) = restore_storage();

    let user = is_logged_in(&user_id)?;
    let challenge = challenges.get_mut(&challenge_id).ok_or("Challenge not found")?;

    // Check if user has completed required courses and quizzes
    let has_completed_courses = challenge.required_courses.iter().all(|course| user.completed_courses.contains(course));
    let has_passed_quizzes = challenge.required_quizzes.iter().all(|quiz| user.passed_quizzes.contains(quiz));
    
    if has_completed_courses && has_passed_quizzes {
        // Update user's challenges
        users.get_mut(&user_id).unwrap().challenges_completed.push(challenge_id.clone());
        
        // Update challenge participants
        challenge.participants.push(user_id.clone());
        
        // Reward user
        users.get_mut(&user_id).unwrap().tokens += challenge.reward_tokens;

        // Create notification for challenge participation
        let notification = Notification {
            user_id: user_id.clone(),
            message: format!("You have successfully participated in the challenge: {}", challenge.description),
            timestamp: ic_cdk::api::time() as i64,
            notification_type: NotificationType::ChallengeStarted,
        };

        notifications.entry(user_id.clone()).or_default().push(notification);

        save_storage(
            users,
            HashMap::<String, UserFootprint>::new(), 
            HashMap::<String, Quiz>::new(), 
            challenges, // Save updated challenges
            HashMap::<String, Token>::new(),
            Vec::<String>::new(),
            HashMap::<String, Course>::new(), // Empty courses
            notifications, // Save notifications
            Vec::<Feedback>::new(), // Empty feedback
        )?;
        
        log_action(&format!("User {} participated in challenge {}", user_id, challenge_id));
        return Ok(format!("Successfully participated in challenge: {}", challenge.description));
    }
    Err("You have not completed the required courses or quizzes to participate in this challenge".to_string())
}

// Add a Challenge
#[update]
fn add_challenge(description: String, reward_tokens: u32, required_courses: Vec<String>, required_quizzes: Vec<String>) -> Result<String, String> {
    let (mut challenges, _, _, _, _, _, _, notifications, _) = restore_storage();

    let challenge_id = format!("challenge_{}", challenges.len() + 1); // Simple ID generation
    challenges.insert(challenge_id.clone(), Challenge { 
        description, 
        reward_tokens, 
        required_courses, 
        required_quizzes,
        participants: Vec::new(),
    });

    // Create notification for new challenge
    let message = format!("A new challenge has been added: {}", description);
    send_notification("all", message, NotificationType::ChallengeAdded); // Send to all users

    save_storage(
        HashMap::<String, User>::new(),
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        challenges, // Save updated challenges
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        HashMap::<String, Course>::new(), // Empty courses
        notifications,
        Vec::<Feedback>::new(), // Empty feedback
    )?;
    
    log_action(&format!("Challenge {} added", challenge_id));
    Ok(format!("Challenge added: {}", challenge_id))
}


// Create Course
#[update]
fn create_course(title: String, levels: HashMap<u32, Quiz>, educational_resources: Vec<String>) -> Result<String, String> {
    let (mut courses, _, _, _, _, _, _, mut notifications, _) = restore_storage();

    if courses.contains_key(&title) {
        return Err("Course already exists".to_string());
    }

    courses.insert(title.clone(), Course { 
        title: title.clone(), 
        levels, 
        educational_resources 
    });

    // Create notification for new course
    let notification = Notification {
        user_id: "all".to_string(), // Notify all users
        message: format!("A new course has been created: {}", title),
        timestamp: ic_cdk::api::time() as i64,
    };

    // Add notification for all users
    notifications.entry("all".to_string()).or_default().push(notification);

    save_storage(
        HashMap::<String, User>::new(),
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        courses, // Save updated courses
        notifications, // Save notifications
        Vec::<Feedback>::new(), // Empty feedback
    )?;
    
    log_action(&format!("Course {} created", title));
    Ok(format!("Course created: {}", title))
}
// Add Quiz
#[update]
fn add_quiz(title: String, level: u32, questions: Vec<String>, options: Vec<Vec<String>>, correct_answers: Vec<String>, reward: u32) -> Result<String, String> {
    let (mut quizzes, _, _, _, _, _, _, mut notifications, _) = restore_storage();

    if quizzes.contains_key(&title) {
        return Err("Quiz already exists".to_string());
    }

    quizzes.insert(title.clone(), Quiz { 
        level, 
        questions, 
        options, 
        correct_answers, 
        reward,
    });

    save_storage(
        HashMap::<String, User>::new(),
        HashMap::<String, UserFootprint>::new(), 
        quizzes, // Save updated quizzes
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        HashMap::<String, Course>::new(), // Empty courses
        notifications, // Save notifications
        Vec::<Feedback>::new(), // Empty feedback
    )?;
    
    log_action(&format!("Quiz {} added", title));
    Ok(format!("Quiz added: {}", title))
}

// Pass Quiz
#[update]
fn pass_quiz(user_id: String, quiz_title: String) -> Result<String, String> {
    let (mut users, quizzes, _, _, _, _, _, _, _) = restore_storage();

    let user = is_logged_in(&user_id)?;

    if let Some(quiz) = quizzes.get(&quiz_title) {
        // Logic to determine if the user passed the quiz
        // (Here, you should implement the logic to evaluate the user's answers)
        // For simplicity, let's say the user passes every quiz they attempt
        users.get_mut(&user_id.clone()).unwrap().passed_quizzes.push(quiz_title.clone()); // Clone user_id
        users.get_mut(&user_id).unwrap().tokens += quiz.reward;

        save_storage(
            users,
            HashMap::<String, UserFootprint>::new(), 
            quizzes, // Save updated quizzes
            HashMap::<String, Challenge>::new(), 
            HashMap::<String, Token>::new(),
            Vec::<String>::new(),
            HashMap::<String, Course>::new(), // Empty courses
            HashMap::<String, Vec<Notification>>::new(), // Empty notifications
            Vec::<Feedback>::new(), // Empty feedback
        )?;
        
        log_action(&format!("User {} passed quiz {}", user_id, quiz_title));
        return Ok(format!("Successfully passed quiz: {}", quiz_title));
    }
    Err("Quiz not found".to_string())
}

fn send_notification(user_id: &str, message: String, notification_type: NotificationType) {
    let (_, _, _, _, _, _, _, mut notifications, _) = restore_storage();

    let notification = Notification {
        user_id: user_id.to_string(),
        message,
        timestamp: ic_cdk::api::time() as i64,
        notification_type,
    };

    notifications.entry(user_id.to_string()).or_default().push(notification);
    
    // Save updated notifications
    save_storage(
        HashMap::<String, User>::new(),
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        HashMap::<String, Course>::new(), // Empty courses
        notifications,
        Vec::<Feedback>::new(), // Empty feedback
    ).unwrap(); // Handle error appropriately
}


#[query]
fn get_notifications(user_id: String) -> Result<Vec<Notification>, String> {
    let (_, _, _, _, _, _, _, notifications, _) = restore_storage();

    // Retrieve notifications for the user
    notifications.get(&user_id).cloned().ok_or("No notifications found".to_string())
}


// Collect Feedback
#[update]
fn collect_feedback(user_id: String, feedback: String) -> Result<String, String> {
    let (_, _, _, _, _, _, _, _, mut feedbacks) = restore_storage();

    feedbacks.push(Feedback { user_id, feedback });
    save_storage(
        HashMap::<String, User>::new(),
        HashMap::<String, UserFootprint>::new(), 
        HashMap::<String, Quiz>::new(), 
        HashMap::<String, Challenge>::new(), 
        HashMap::<String, Token>::new(),
        Vec::<String>::new(),
        HashMap::<String, Course>::new(), // Empty courses
        HashMap::<String, Vec<Notification>>::new(), // Empty notifications
        feedbacks, // Save updated feedback
    )?;
    
    log_action(&format!("User {} submitted feedback", user_id));
    Ok("Feedback submitted".to_string())
}
