use deadpool_redis::{Connection, redis::AsyncTypedCommands};
use anyhow::{Error, Result};


pub async fn is_logged_in_with_username(username: &str, connection: &mut Connection) -> Result<bool> {
    let exists = match connection.exists::<_>(username).await {
        Ok(e) => e,
        Err(e) => return Err(e.into())
    };
    Ok(exists)
}

pub async fn is_logged_in_with_token(token: &str, connection: &mut Connection) -> Result<bool> {
    let exists = match connection.exists::<_>(token).await {
        Ok(e) => e,
        Err(e) => return Err(e.into())
    };
    Ok(exists)
}

pub async fn login_user(username: &str, token: &str, ttl_seconds: usize, connection: &mut Connection) -> Result<()> {
    if let Err(e) = connection.set_ex::<_, _>(username, token, ttl_seconds as u64).await {
        return Err(e.into());
    }

    if let Err(e) = connection.set_ex::<_, _>(token, username, ttl_seconds as u64).await {
        return Err(e.into());
    }
    Ok(())
}

pub async fn logout_user(token: &str, connection: &mut Connection) -> Result<()> {
    let username = match connection.get_del(token).await {
        Ok(u) => u,
        Err(e) => return Err(e.into())
    };

    if let None = username {
        return Err(Error::msg("Error: Username does not exist"));
    }

    if let Err(e) = connection.get_del::<_>(username.unwrap()).await {
        return Err(e.into());
    }
    Ok(())
}