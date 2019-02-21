pub type UserID = u64;
pub type AuthToken = String;

pub enum AuthState {
    Disconnected,
    Connected {
        user_id: UserID,
        level: AuthLevel,
        token: AuthToken,
    },
}

pub enum AuthLevel {
    Admin,
    TeamLeader,
}

#[derive(Serialize, Deserialize)]
pub struct JWTClaims {
    iss: String,
    iat: u64,
    exp: u64,
    user: UserID,
}