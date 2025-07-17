use tokio::sync::mpsc::Receiver;

use crate::scan::ScanResults;

pub async fn report(mut scan_rx: Receiver<ScanResults>) {
    loop {
        tokio::select! {
            scan_res = scan_rx.recv() => {
                    println!("RES: {:?}", scan_res);
                }
        }
    }
}
