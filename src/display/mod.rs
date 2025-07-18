mod structs;

pub use structs::*;

use crate::report::report;
use crate::tui::App;

pub async fn display(tui: bool, args: DisplayArgs) {
    if tui {
        let terminal = ratatui::init();

        let app = App::new(
            args.channels.spectrum_rx,
            args.channels.freq_rx,
            args.channels.scan_rx,
            args.params.rate,
            args.params.start_freq,
        );
        let _ = app.run(terminal);
        ratatui::restore();
    } else {
        report(args.channels.scan_rx).await;
    }
}
