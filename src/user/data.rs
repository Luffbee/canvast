use regex::Regex;

use std::ops::RangeInclusive;

use super::{UserError, UserResult};

pub type Username = String;

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub name: Username,
}

impl User {
    pub fn validate(&self) -> UserResult<()> {
        // validate name
        const LEN: RangeInclusive<usize> = 1..=64;
        const PAT: &str = r"^[a-zA-Z][-.@\w]*$";
        lazy_static! {
            static ref RE: Regex = Regex::new(PAT).unwrap();
        }
        validate_length("name", &self.name, &LEN)?;
        validate_pattern("name", &self.name, &RE)?;
        Ok(())
    }
}

#[derive(Deserialize, Clone)]
pub struct WithPassword {
    #[serde(flatten)]
    pub user: User,
    pub password: String,
}

impl WithPassword {
    pub fn validate(&self) -> UserResult<()> {
        self.user.validate()?;
        // validate password
        const LEN: RangeInclusive<usize> = 6..=32;
        const PAT: &str = r"^[-.@\w]*$";
        lazy_static! {
            static ref RE: Regex = Regex::new(PAT).unwrap();
        }
        validate_length("password", &self.password, &LEN)?;
        validate_pattern("password", &self.password, &RE)
    }
}

fn validate_length(name: &str, value: &str, len: &RangeInclusive<usize>) -> Result<(), UserError> {
    if !len.contains(&value.len()) {
        Err(UserError::InvalidData(format!(
            "{} must have {} to {} characters",
            name,
            len.start(),
            len.end()
        )))
    } else {
        Ok(())
    }
}

fn validate_pattern(name: &str, value: &str, re: &Regex) -> UserResult<()> {
    if !re.is_match(value) {
        Err(UserError::InvalidData(format!(
            "{} must match pattern: {}",
            name,
            re.as_str()
        )))
    } else {
        Ok(())
    }
}
