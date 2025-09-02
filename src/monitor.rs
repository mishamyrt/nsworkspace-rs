use std::sync::mpsc;

use objc2::rc::{Retained};
use objc2::runtime::{ProtocolObject, Sel};
use objc2::{
    define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
    NSApplication, NSApplicationDelegate, NSWorkspace,
    NSWorkspaceDidActivateApplicationNotification,
    NSWorkspaceDidDeactivateApplicationNotification,
    NSWorkspaceDidHideApplicationNotification,
    NSWorkspaceDidLaunchApplicationNotification,
    NSWorkspaceDidTerminateApplicationNotification,
    NSWorkspaceDidUnhideApplicationNotification, NSWorkspaceDidWakeNotification,
    NSWorkspaceScreensDidSleepNotification, NSWorkspaceScreensDidWakeNotification,
    NSWorkspaceWillPowerOffNotification, NSWorkspaceWillSleepNotification,
};
use objc2_foundation::{
    NSNotification, NSObject, NSObjectNSThreadPerformAdditions, NSObjectProtocol,
    NSString,
};

use crate::events::{Event, NotificationListener};
use crate::parse::{app_identifier_from_notification, running_application_identifier};

#[derive(Debug)]
#[allow(unused)]
struct Ivars {
    events: Box<mpsc::Sender<Event>>,
}

define_class!(
    // SAFETY:
    // - The superclass NSObject does not have any subclassing requirements.
    // - `AppDelegate` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = Ivars]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSApplicationDelegate for AppDelegate {}

    impl AppDelegate {
        #[unsafe(method(didActivateApplication:))]
        fn did_activate_application(&self, notification: &NSNotification) {
            self.send_app_event(notification, Event::DidActivateApplication);
        }

        #[unsafe(method(didDeactivateApplication:))]
        fn did_deactivate_application(&self, notification: &NSNotification) {
            self.send_app_event(notification, Event::DidDeactivateApplication);
        }

        #[unsafe(method(didHideApplication:))]
        fn did_hide_application(&self, notification: &NSNotification) {
            self.send_app_event(notification, Event::DidHideApplication);
        }

        #[unsafe(method(didUnhideApplication:))]
        fn did_unhide_application(&self, notification: &NSNotification) {
            self.send_app_event(notification, Event::DidUnhideApplication);
        }

        #[unsafe(method(didLaunchApplication:))]
        fn did_launch_application(&self, notification: &NSNotification) {
            self.send_app_event(notification, Event::DidLaunchApplication);
        }

        #[unsafe(method(didTerminateApplication:))]
        fn did_terminate_application(&self, notification: &NSNotification) {
            self.send_app_event(notification, Event::DidTerminateApplication);
        }

        #[unsafe(method(didSleep:))]
        fn did_sleep(&self, _: &NSNotification) {
            self.send_event(Event::DidSleep);
        }

        #[unsafe(method(didWake:))]
        fn did_wake(&self, _: &NSNotification) {
            self.send_event(Event::DidWake);
        }

        #[unsafe(method(didPowerOff:))]
        fn did_power_off(&self, _: &NSNotification) {
            self.send_event(Event::DidPowerOff);
        }

        #[unsafe(method(didScreenSleep:))]
        fn did_screen_sleep(&self, _: &NSNotification) {
            self.send_event(Event::DidScreenSleep);
        }

        #[unsafe(method(didScreenWake:))]
        fn did_screen_wake(&self, _: &NSNotification) {
            self.send_event(Event::DidScreenWake);
        }
    }
);

impl AppDelegate {
    fn new(mtm: MainThreadMarker, events: mpsc::Sender<Event>) -> Retained<Self> {
        let this = Self::alloc(mtm);
        let events = Box::new(events);
        let this = this.set_ivars(Ivars { events });

        unsafe { msg_send![super(this), init] }
    }

    unsafe fn get_active_application() -> Option<String> {
        unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            let active_application = workspace.frontmostApplication().unwrap();
            running_application_identifier(&active_application)
        }
    }

    fn send_app_event(
        &self,
        notification: &NSNotification,
        event_mapper: impl FnOnce(String) -> Event,
    ) {
        let identifier_result =
            unsafe { app_identifier_from_notification(notification) };
        let event = match identifier_result {
            Ok(identifier) => event_mapper(identifier),
            Err(e) => Event::Error(e),
        };

        self.send_event(event);
    }

    fn send_event(&self, event: Event) {
        if let Err(e) = self.ivars().events.send(event) {
            panic!("unable to send event: {e:?}");
        }
    }

    /// Returns the selector and name for the given notification listener.
    unsafe fn notification_handler(
        event: NotificationListener,
    ) -> (Sel, &'static NSString) {
        match event {
            // Activation/focus
            NotificationListener::DidActivateApplication => (
                sel!(didActivateApplication:),
                NSWorkspaceDidActivateApplicationNotification,
            ),
            NotificationListener::DidDeactivateApplication => (
                sel!(didDeactivateApplication:),
                NSWorkspaceDidDeactivateApplicationNotification,
            ),

            // Application lifecycle
            NotificationListener::DidLaunchApplication => (
                sel!(didLaunchApplication:),
                NSWorkspaceDidLaunchApplicationNotification,
            ),
            NotificationListener::DidTerminateApplication => (
                sel!(didTerminateApplication:),
                NSWorkspaceDidTerminateApplicationNotification,
            ),

            // Visibility
            NotificationListener::DidHideApplication => (
                sel!(didHideApplication:),
                NSWorkspaceDidHideApplicationNotification,
            ),
            NotificationListener::DidUnhideApplication => (
                sel!(didUnhideApplication:),
                NSWorkspaceDidUnhideApplicationNotification,
            ),

            // Power
            NotificationListener::DidSleep => {
                (sel!(didSleep:), NSWorkspaceWillSleepNotification)
            }
            NotificationListener::DidWake => {
                (sel!(didWake:), NSWorkspaceDidWakeNotification)
            }
            NotificationListener::DidPowerOff => {
                (sel!(didPowerOff:), NSWorkspaceWillPowerOffNotification)
            }

            // Screens
            NotificationListener::DidScreenSleep => (
                sel!(didScreenSleep:),
                NSWorkspaceScreensDidSleepNotification,
            ),
            NotificationListener::DidScreenWake => {
                (sel!(didScreenWake:), NSWorkspaceScreensDidWakeNotification)
            }
            _ => {
                unreachable!();
            }
        }
    }
}

/// `NSWorkspace` Monitor owns the `AppDelegate` instance and is responsible for starting and stopping the listening.
pub struct Monitor {
    delegate: Retained<AppDelegate>,
    mtm: MainThreadMarker,
}

impl Monitor {
    /// Creates a new `Monitor` instance.
    pub fn new() -> Option<(Self, mpsc::Receiver<Event>)> {
        let (events, events_rx) = mpsc::channel();

        let mtm = MainThreadMarker::new()?;
        let delegate = AppDelegate::new(mtm, events);

        Some((Self { delegate, mtm }, events_rx))
    }

    /// Starts the `NSWorkspace` run loop.
    pub fn run(&self) {
        let app = NSApplication::sharedApplication(self.mtm);
        let object = ProtocolObject::from_ref(&*self.delegate);
        app.setDelegate(Some(object));
        app.run();
    }

    /// Stops the `NSWorkspace` run loop.
    pub fn stop(&self) {
        let app = NSApplication::sharedApplication(self.mtm);
        unsafe {
            app.performSelectorOnMainThread_withObject_waitUntilDone(
                sel!(terminate:),
                None,
                false,
            );
        }
    }

    /// Returns the bundle identifier of the currently active application.
    pub fn get_active_application(&self) -> Option<String> {
        unsafe { AppDelegate::get_active_application() }
    }

    /// Subscribes to the given notification listeners.
    pub fn subscribe(&self, listeners: NotificationListener) {
        unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            let center = workspace.notificationCenter();

            for (_, flag) in NotificationListener::flags() {
                if !listeners.contains(*flag) {
                    continue;
                }
                let (selector, name) = AppDelegate::notification_handler(*flag);
                center.addObserver_selector_name_object(
                    self.delegate.as_ref(),
                    selector,
                    Some(name),
                    Some(workspace.as_ref()),
                );
            }
        }
    }
}
