use std::sync::OnceLock;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

/// JWT signing secret, read once from the environment.
/// `main` verifies JWT_SECRET is set at startup, so the expect here cannot fire at runtime.
pub fn jwt_secret() -> &'static [u8] {
    static SECRET: OnceLock<String> = OnceLock::new();
    SECRET
        .get_or_init(|| std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"))
        .as_bytes()
}

/// Hash a password with Argon2id and a random salt.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

/// Verify a password against a stored Argon2 hash.
/// Returns false for wrong passwords and for unparseable stored hashes.
pub fn verify_password(password: &str, password_hash: &str) -> bool {
    PasswordHash::new(password_hash)
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}
