use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = match self {
            Day::Monday => "Mon".into(),
            Day::Tuesday => "Tue".into(),
            Day::Wednesday => "Wed".into(),
            Day::Thursday => "Thu".into(),
            Day::Friday => "Fri".into(),
            Day::Saturday => "Sat".into(),
            Day::Sunday => "Sun".into(),
        };
        write!(f, "{}", str)
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Recurrence {
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
}

impl Display for Recurrence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rec = vec![];
        if self.monday {
            rec.push(Day::Monday.to_string())
        }
        if self.tuesday {
            rec.push(Day::Tuesday.to_string())
        }
        if self.wednesday {
            rec.push(Day::Wednesday.to_string())
        }
        if self.thursday {
            rec.push(Day::Thursday.to_string())
        }
        if self.friday {
            rec.push(Day::Friday.to_string())
        }
        if self.saturday {
            rec.push(Day::Saturday.to_string())
        }
        if self.sunday {
            rec.push(Day::Sunday.to_string())
        }
        write!(f, "{}", rec.join(", "))
    }
}
