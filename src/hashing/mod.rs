pub mod hashing {
    use argon2::{
        password_hash::{
            rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        },
        Argon2,
    };

    pub fn hash_my_password(password: String) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        let parsed_hash = PasswordHash::new(&password_hash)?;

        Ok(parsed_hash.to_string())
    }
}
