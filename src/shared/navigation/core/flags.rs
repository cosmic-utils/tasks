use cosmic::cosmic_config::Config;

use crate::{config::AppConfig, shared::store::Store};

#[derive(Clone, Debug)]
pub struct Flags {
    pub handler: Config,
    pub config: AppConfig,
    pub store: Store,
}
