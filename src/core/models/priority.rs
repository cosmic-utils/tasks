use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum Priority {
    #[default]
    Low = 0,
    Normal = 1,
    High = 2,
}

impl From<i32> for Priority {
    fn from(value: i32) -> Self {
        match value {
            0 => Priority::Low,
            1 => Priority::Normal,
            2 => Priority::High,
            _ => panic!("Invalid value for Priority"),
        }
    }
}

impl From<Priority> for i32 {
    fn from(value: Priority) -> Self {
        match value {
            Priority::Low => 0,
            Priority::Normal => 1,
            Priority::High => 2,
        }
    }
}

impl Priority {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Priority::Low => "LOW",
            Priority::Normal => "NORMAL",
            Priority::High => "HIGH",
        }
    }

    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "LOW" => Some(Self::Low),
            "NORMAL" => Some(Self::Normal),
            "HIGH" => Some(Self::High),
            _ => None,
        }
    }
}
