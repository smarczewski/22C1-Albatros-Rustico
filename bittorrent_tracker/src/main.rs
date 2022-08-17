use tracker::{constants::TRACKER_ADDRESS, tracker::Tracker};

fn main() {
    if let Ok(tracker) = Tracker::new(TRACKER_ADDRESS) {
        tracker.run();
    } else {
        println!("Error: Cannot bind to address");
    }
}
