use crate::models::User;
use crate::ports::{Database, Hasher};

pub async fn register(
    user: User,
    db: impl Database,
    hasher: impl Hasher,
) -> Result<(), Box<dyn std::error::Error>> {
    let hashed_passsowrd = hasher.hash_password(user.password).await;

    db.add_user(User {
        username: user.username,
        password: hashed_passsowrd,
    })
    .await?;
    Ok(())
}

pub async fn login(user: User, db: impl Database, hasher: impl Hasher) -> Result<User, String> {
    let u = user.clone();
    let a = db.get_user(user.username).await;
    if let Err(_) = a {
        return Err(format!("User Not Found"));
    }

    match a.unwrap() {
        None => Err(format!("User Not Found")),
        Some(found_user) => {
            if hasher
                .compare_password(u.password, found_user.password.clone())
                .await
            {
                Ok(found_user.clone())
            } else {
                Err(format!("Incorrect password"))
            }
        }
    }
}
