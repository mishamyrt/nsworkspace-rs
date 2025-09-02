use bitmask_enum::bitmask;

use crate::NSWorkspaceError;

#[derive(Debug, Clone)]
pub enum Event {
    // Activation/focus
    DidActivateApplication(String),
    DidDeactivateApplication(String),

    // Application lifecycle
    DidLaunchApplication(String),
    DidTerminateApplication(String),

    // Visibility
    DidHideApplication(String),
    DidUnhideApplication(String),

    // Power
    DidSleep,
    DidWake,
    DidPowerOff,

    // Screens
    DidScreenSleep,
    DidScreenWake,

    Error(NSWorkspaceError),
}

unsafe impl Send for Event {}
unsafe impl std::marker::Sync for Event {}

#[bitmask]
#[bitmask_config(flags_iter)]
pub enum NotificationListener {
    // Activation/focus
    DidActivateApplication,
    DidDeactivateApplication,

    // Application lifecycle
    DidLaunchApplication,
    DidTerminateApplication,

    // Visibility
    DidHideApplication,
    DidUnhideApplication,

    // Power
    DidSleep,
    DidWake,
    DidPowerOff,

    // Screens
    DidScreenSleep,
    DidScreenWake,
}
