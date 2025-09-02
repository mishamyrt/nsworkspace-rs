use std::thread;

use nsworkspace::{NotificationListener, Monitor};

#[allow(clippy::print_stderr, clippy::print_stdout)]
fn main() {
    let Some((monitor, events)) = Monitor::new() else {
        eprintln!("Failed to create NSWorkspace monitor");
        return;
    };
    let active_application = monitor.get_active_application();
    println!("Active application: {active_application:?}");

    thread::spawn(|| {
        for event in events {
            println!("Event: {event:?}");
        }
    });

    let mut listeners: NotificationListener =
        NotificationListener::DidActivateApplication;
    for (_, flag) in NotificationListener::flags() {
        listeners |= *flag;
    }

    monitor.subscribe(listeners);
    monitor.run();
}
