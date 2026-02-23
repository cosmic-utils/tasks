use crate::{config::AppConfig, services::store::Store};
use cosmic::cosmic_config::Config;

#[derive(Clone, Debug)]
pub struct Flags {
    pub handler: Config,
    pub config: AppConfig,
    pub store: Store,
}
