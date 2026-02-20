use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ListAction {
    Add(Uuid),
    Delete(Uuid),
    Rename(Uuid, String),
    Export(Uuid),
}
