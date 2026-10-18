use crate::fl;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextPage {
    About,
    #[allow(unused)]
    Accounts,
    Settings,
    TaskDetails,
}

impl ContextPage {
    pub fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::Accounts => fl!("accounts"),
            Self::Settings => fl!("settings"),
            Self::TaskDetails => fl!("details"),
        }
    }
}
