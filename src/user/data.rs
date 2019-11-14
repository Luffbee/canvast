use regex::Regex;

use std::ops::RangeInclusive;

pub type Username = String;

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub name: Username,
    #[serde(default)]
    pub intro: String,
}

impl User {
    pub fn validate(&self) -> Result<(), String> {
        {
            // validate name
            const LEN: RangeInclusive<usize> = 1..=64;
            const PAT: &str = r"^[a-zA-Z][-.@\w]*$";
            lazy_static! {
                static ref RE: Regex = Regex::new(PAT).unwrap();
            }
            validate_length("name", &self.name, &LEN)?;
            validate_pattern("name", &self.name, &RE)?;
        }
        {
            // validate introduction
            const LEN: RangeInclusive<usize> = 0..=128;
            validate_length("introduction", &self.intro, &LEN)?;
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct WithPassword {
    #[serde(flatten)]
    pub user: User,
    pub password: String,
}

impl WithPassword {
    pub fn validate(&self) -> Result<(), String> {
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Location {
    pub location: String,
}

impl Location {
    pub fn validate(&self) -> Result<(), String> {
        const PAT: &str = r"^-?\d+,-?\d+$";
        lazy_static! {
            static ref RE: Regex = Regex::new(PAT).unwrap();
        }
        validate_pattern("location", &self.location, &RE)
    }
}

fn validate_length(name: &str, value: &str, len: &RangeInclusive<usize>) -> Result<(), String> {
    if !len.contains(&value.len()) {
        Err(format!(
            "{} must have {} to {} characters",
            name,
            len.start(),
            len.end()
        ))
    } else {
        Ok(())
    }
}

fn validate_pattern(name: &str, value: &str, re: &Regex) -> Result<(), String> {
    if !re.is_match(value) {
        Err(format!("{} must match pattern: {}", name, re.as_str()))
    } else {
        Ok(())
    }
}
