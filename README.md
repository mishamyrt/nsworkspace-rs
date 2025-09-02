# NSWorkspace Rust Library

A library for listening to NSWorkspace notifications. It provides a high-level interface for listening to notifications.

### Features

- High-level Rust API over AppKit `NSWorkspace` notifications
- Rich `Event` enum: activate/deactivate, launch/terminate, hide/unhide
- System events: sleep/wake/power off, screen sleep/wake
- Selective subscriptions via `NotificationListener` bitmask flags
- Channel-based delivery with `std::sync::mpsc`
- Helper to read the current frontmost app bundle identifier

## Usage

First, add package to your `Cargo.toml`:
```toml
[dependencies]
nsworkspace = "0.1.0"
```

Then, you can use the library to listen to NSWorkspace notifications.

```rust
use std::thread;
use nsworkspace::{events::NotificationListener, monitor::Monitor};

fn main() {
    let Some((monitor, events)) = Monitor::new() else {
        eprintln!("NSWorkspace monitor must be created on the main thread");
        return;
    };

    // Consume events on a background thread
    thread::spawn(move || {
        for event in events {
            println!("Event: {event:?}");
        }
    });

    // Subscribe to some notifications
    let listeners = NotificationListener::DidActivateApplication
        | NotificationListener::DidLaunchApplication
        | NotificationListener::DidTerminateApplication;
    monitor.subscribe(listeners);

    // Optional: print current frontmost app
    println!("Active application: {:?}", monitor.get_active_application());

    // Start the AppKit run loop (blocking)
    monitor.run();
}
```

### Notes

- macOS only (uses AppKit `NSWorkspace`)
- Requires Rust 1.80+
