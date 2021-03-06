use parking_lot::RwLock;
use rand::{thread_rng, Rng};

use std::collections::HashMap;
use time::{Duration, Tm};

use crate::paint::PixelPos;

use super::data::*;
use super::{UserError, UserResult};

const TIMEOUT: i64 = 30; // 30 days

type Token = String;

pub struct UserDB(RwLock<SimpleDB>);

impl UserDB {
    pub fn new() -> Self {
        Self(RwLock::new(SimpleDB::new()))
    }

    pub async fn new_user(&self, user: WithPassword) -> UserResult<()> {
        self.0.write().new_user(user)
    }

    pub async fn login(&self, user: &WithPassword) -> UserResult<(Token, Tm)> {
        self.0.write().login(user)
    }

    pub async fn check_token(&self, token: &str) -> UserResult<Username> {
        self.0.read().check_token(token)
    }

    pub async fn logout(&self, token: &str) -> UserResult<()> {
        self.0.write().logout(token)
    }

    pub async fn set_location(&self, name: Username, loc: PixelPos) -> UserResult<()> {
        self.0.write().set_location(name, loc)
    }

    pub async fn get_location(&self, name: &str) -> UserResult<PixelPos> {
        self.0.read().get_location(name)
    }
}

struct TokenInfo {
    name: Username,
    expire: Tm,
}

struct SimpleDB {
    users: HashMap<String, WithPassword>,
    tokens: HashMap<Token, TokenInfo>,
    locations: HashMap<String, PixelPos>,
}

impl SimpleDB {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            tokens: HashMap::new(),
            locations: HashMap::new(),
        }
    }

    fn has_user(&self, name: &str) -> UserResult<bool> {
        match self.users.get(name) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn new_user(&mut self, user: WithPassword) -> UserResult<()> {
        if self.has_user(&user.user.name)? {
            return Err(UserError::UserAlreadyExist);
        }
        self.users.insert(user.user.name.clone(), user);
        Ok(())
    }

    fn login(&mut self, user: &WithPassword) -> UserResult<(Token, Tm)> {
        let u = self
            .users
            .get(&user.user.name)
            .ok_or(UserError::LoginFailed)?;
        if u.password != user.password {
            return Err(UserError::LoginFailed);
        }

        // check passed
        let token = {
            let buf: &mut [u8] = &mut [0u8; 64];
            thread_rng().fill(buf);
            base64::encode(buf)
        };
        let exp = time::now() + Duration::days(TIMEOUT);
        self.tokens.insert(
            token.clone(),
            TokenInfo {
                name: user.user.name.clone(),
                expire: exp,
            },
        );
        Ok((token, exp))
    }

    fn check_token(&self, token: &str) -> UserResult<Username> {
        match self.tokens.get(token) {
            None => Err(UserError::BadToken),
            Some(info) => {
                if info.expire < time::now() {
                    Err(UserError::BadToken)
                } else {
                    Ok(info.name.clone())
                }
            }
        }
    }

    fn logout(&mut self, token: &str) -> UserResult<()> {
        match self.tokens.get(token) {
            None => Err(UserError::BadToken),
            Some(_) => {
                self.tokens.remove(token);
                Ok(())
            }
        }
    }

    fn set_location(&mut self, name: Username, loc: PixelPos) -> UserResult<()> {
        self.locations.insert(name, loc);
        Ok(())
    }

    fn get_location(&self, name: &str) -> UserResult<PixelPos> {
        match self.locations.get(name) {
            None => Ok(PixelPos::default()),
            Some(loc) => Ok(*loc),
        }
    }
}
