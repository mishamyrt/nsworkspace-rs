mod monitor;
mod events;
pub mod parse;

use objc2::runtime::AnyObject;
use thiserror::Error;

use objc2_foundation::{NSDictionary, NSString};

pub(crate) type UserInfo = NSDictionary<NSString, AnyObject>;

pub use monitor::Monitor;
pub use events::{Event, NotificationListener};

#[derive(Error, Clone, Debug)]
pub enum NSWorkspaceError {
    #[error("Failed to get application key")]
    GetApplicationKey,
    #[error("Failed to get bundle identifier")]
    GetBundleIdentifier,
    #[error("Failed to get user info")]
    GetUserInfo,
}
