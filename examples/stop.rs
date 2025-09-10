use std::thread;

use nsworkspace::{Monitor, NotificationListener};

fn main() {
    let Some((monitor, events, stop_tx)) = Monitor::new() else {
        eprintln!("Failed to create NSWorkspace monitor");
        return;
    };

    ctrlc::set_handler(move || {
        println!("Ctrl+C pressed, stopping NSWorkspace monitor");
        let _ = stop_tx.send(());
    })
    .expect("failed to set Ctrl+C handler");

    thread::spawn(move || loop {
        for event in &events {
            println!("Event: {event:?}");
        }
    });

    monitor.subscribe(
        NotificationListener::DidTerminateApplication
            | NotificationListener::DidHideApplication
            | NotificationListener::DidUnhideApplication
            | NotificationListener::DidActivateApplication
            | NotificationListener::DidDeactivateApplication,
    );

    monitor.run();
}
