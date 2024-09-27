use std::fmt::Display;

#[derive(Debug)]
pub enum State {
    Unknown(String),
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Stopped,
    Tracing,
    Dead,
    Idle,
}

impl Default for State {
    fn default() -> Self {
        State::Unknown(String::default())
    }
}

impl From<&str> for State {
    fn from(value: &str) -> Self {
        match value {
            "R" => State::Running,
            "S" => State::Sleeping,
            "D" => State::Waiting,
            "Z" => State::Zombie,
            "T" => State::Stopped,
            "t" => State::Tracing,
            "X" => State::Dead,
            "I" => State::Idle,
            s => State::Unknown(s.to_string()),
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Unknown(s) => write!(f, "Unknown({})", s),
            State::Running => write!(f, "R"),
            State::Sleeping => write!(f, "S"),
            State::Waiting => write!(f, "D"),
            State::Zombie => write!(f, "Z"),
            State::Stopped => write!(f, "T"),
            State::Tracing => write!(f, "t"),
            State::Dead => write!(f, "X"),
            State::Idle => write!(f, "I"),
        }
    }
}
