use c122_albatros_rustico::bittorrent::run_bittorrent;
use c122_albatros_rustico::gui::gui_model::UserInterface;
use std::sync::mpsc::channel;
use std::thread;

fn main() {
    let (tx_gui, rx_gui) = channel();
    let bt_thread = thread::spawn(move || {
        run_bittorrent(rx_gui);
    });

    UserInterface::run(tx_gui);

    if bt_thread.join().is_err() {
        println!("Error during bittorrent thread joining");
    }
}
