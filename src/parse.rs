use objc2::rc::Retained;
use objc2_app_kit::{NSRunningApplication, NSWorkspaceApplicationKey};
use objc2_foundation::NSNotification;

use crate::{NSWorkspaceError, UserInfo};

pub(crate) unsafe fn running_application_identifier(
    app: &NSRunningApplication,
) -> Option<String> {
    app.bundleIdentifier().map(|s| s.to_string())
}

pub(crate) unsafe fn app_identifier_from_notification(
    notification: &NSNotification,
) -> Result<String, NSWorkspaceError> {
    unsafe {
        let Some(user_info_any) = notification.userInfo() else {
            return Err(NSWorkspaceError::GetUserInfo);
        };
        let user_info: Retained<UserInfo> = Retained::cast_unchecked(user_info_any);
        let Some(app_any) = user_info.objectForKey(NSWorkspaceApplicationKey) else {
            return Err(NSWorkspaceError::GetApplicationKey);
        };
        let Ok(app) = app_any.downcast::<NSRunningApplication>() else {
            return Err(NSWorkspaceError::GetApplicationKey);
        };
        let Some(bundle_id) = app.bundleIdentifier().or_else(|| app.localizedName())
        else {
            return Err(NSWorkspaceError::GetBundleIdentifier);
        };

        Ok(bundle_id.to_string())
    }
}
