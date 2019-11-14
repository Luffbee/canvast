use rand::{thread_rng, Rng};

use std::collections::HashMap;
use std::sync::RwLock;
use time::{Duration, Tm};

use super::data::*;

const TIMEOUT: i64 = 30; // 30 days

type Token = String;

pub trait DB: Send + Sync {
    fn new() -> Self;
    fn get_user(&self, name: &str) -> Result<User, String>;
    fn has_user(&self, name: &str) -> Result<bool, String>;
    fn new_user(&self, user: WithPassword) -> Result<(), String>;
    fn login(&self, user: &WithPassword) -> Result<(Token, Tm), String>;
    fn check_token(&self, token: &str) -> Result<Username, String>;
    fn logout(&self, token: &str) -> Result<(), String>;
    fn set_location(&self, name: Username, loc: Location) -> Result<(), String>;
    fn get_location(&self, name: &str) -> Result<Location, String>;
}

pub struct SharedDB(RwLock<SimpleDB>);

impl DB for SharedDB {
    fn new() -> Self {
        Self(RwLock::new(SimpleDB::new()))
    }

    fn get_user(&self, name: &str) -> Result<User, String> {
        self.0.read().unwrap().get_user(name)
    }

    fn has_user(&self, name: &str) -> Result<bool, String> {
        self.0.read().unwrap().has_user(name)
    }

    fn new_user(&self, user: WithPassword) -> Result<(), String> {
        self.0.write().unwrap().new_user(user)
    }

    fn login(&self, user: &WithPassword) -> Result<(Token, Tm), String> {
        self.0.write().unwrap().login(user)
    }

    fn check_token(&self, token: &str) -> Result<Username, String> {
        self.0.read().unwrap().check_token(token)
    }

    fn logout(&self, token: &str) -> Result<(), String> {
        self.0.write().unwrap().logout(token)
    }

    fn set_location(&self, name: Username, loc: Location) -> Result<(), String> {
        self.0.write().unwrap().set_location(name, loc)
    }

    fn get_location(&self, name: &str) -> Result<Location, String> {
        self.0.read().unwrap().get_location(name)
    }
}

struct TokenInfo {
    name: Username,
    expire: Tm,
}

struct SimpleDB {
    users: HashMap<String, WithPassword>,
    tokens: HashMap<Token, TokenInfo>,
    locations: HashMap<String, Location>,
}

impl SimpleDB {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            tokens: HashMap::new(),
            locations: HashMap::new(),
        }
    }

    fn get_user(&self, name: &str) -> Result<User, String> {
        match self.users.get(name) {
            Some(u) => Ok(u.user.clone()),
            None => Err("user not exist".to_owned()),
        }
    }

    fn has_user(&self, name: &str) -> Result<bool, String> {
        match self.users.get(name) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn new_user(&mut self, user: WithPassword) -> Result<(), String> {
        if self.has_user(&user.user.name)? {
            return Err("user already exist".to_owned());
        }
        println!(
            "{} {}",
            serde_json::to_string(&user.user).unwrap(),
            user.password
        );
        self.users.insert(user.user.name.clone(), user);
        Ok(())
    }

    fn login(&mut self, user: &WithPassword) -> Result<(Token, Tm), String> {
        let u = match self.users.get(&user.user.name) {
            Some(u) => u,
            None => return Err("user not exist".to_owned()),
        };
        if u.password != user.password {
            return Err("invalid password or username".to_owned());
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

    fn check_token(&self, token: &str) -> Result<Username, String> {
        match self.tokens.get(token) {
            None => Err("invalid token".to_owned()),
            Some(info) => {
                if info.expire < time::now() {
                    Err("token expired".to_owned())
                } else {
                    Ok(info.name.clone())
                }
            }
        }
    }

    fn logout(&mut self, token: &str) -> Result<(), String> {
        match self.tokens.get(token) {
            None => Err("invalid token".to_owned()),
            Some(_) => {
                self.tokens.remove(token);
                Ok(())
            }
        }
    }

    fn set_location(&mut self, name: Username, loc: Location) -> Result<(), String> {
        self.locations.insert(name, loc);
        Ok(())
    }

    fn get_location(&self, name: &str) -> Result<Location, String> {
        match self.locations.get(name) {
            None => Ok(Location {
                location: "0,0".to_owned(),
            }),
            Some(loc) => Ok(loc.clone()),
        }
    }
}
