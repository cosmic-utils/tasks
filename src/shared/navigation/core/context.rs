use crate::fl;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextPage {
    About,
    TaskDetails,
    Settings,
    Accounts,
}

impl ContextPage {
    pub fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::Settings => fl!("settings"),
            Self::TaskDetails => fl!("details"),
            Self::Accounts => fl!("accounts"),
        }
    }
}
