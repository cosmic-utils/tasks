mod list;
mod task;
pub use list::List;
pub use task::TrashedTask;
pub use task::*;

/// Zero-sized marker struct to tag the trash nav item.
pub struct TrashMarker;

/// Zero-sized marker struct to tag the favorites nav item.
pub struct FavoritesMarker;
