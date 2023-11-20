#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwtConfig {
    pub secret: JwtSecretConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwtSecretConfig {
    pub current: String,
    pub retired: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
    pub google: GoogleAuthConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogleAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}
