use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Where a `List`/`Task` originates from: created locally, or mirrored from a
/// remote account. `remote_id` is the provider's opaque list/task identifier;
/// the local `Uuid` primary key is generated once on first pull and never
/// regenerated.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskSource {
    #[default]
    Local,
    Google {
        account_id: Uuid,
        remote_id: String,
    },
    Microsoft {
        account_id: Uuid,
        remote_id: String,
    },
}

impl TaskSource {
    pub fn is_local(&self) -> bool {
        matches!(self, TaskSource::Local)
    }

    pub fn account_id(&self) -> Option<Uuid> {
        match self {
            TaskSource::Local => None,
            TaskSource::Google { account_id, .. } | TaskSource::Microsoft { account_id, .. } => {
                Some(*account_id)
            }
        }
    }

    pub fn remote_id(&self) -> Option<&str> {
        match self {
            TaskSource::Local => None,
            TaskSource::Google { remote_id, .. } | TaskSource::Microsoft { remote_id, .. } => {
                Some(remote_id.as_str())
            }
        }
    }
}
