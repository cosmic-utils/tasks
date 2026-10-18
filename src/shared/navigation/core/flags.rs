use cosmic::cosmic_config::Config;

use crate::{config::AppConfig, shared::store::Store};

#[derive(Clone, Debug)]
pub struct Flags {
    pub config: AppConfig,
    pub handler: Config,
    pub store: Store,
}
