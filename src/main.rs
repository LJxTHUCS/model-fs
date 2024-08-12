use km_checker::{CheckLevel, Checker, MockTestPort, StdoutPrinter};
use model_fs::{FileSystem, FsCommander};

fn main() {
    let mock_port = MockTestPort::new(FileSystem::new_bare(0, 0));
    let mut checker = Checker::new(
        FsCommander,
        mock_port,
        StdoutPrinter,
        FileSystem::new_bare(0, 0),
    );
    let mut i = 1;
    loop {
        if let Err(e) = checker.step(CheckLevel::Relaxed, CheckLevel::Strict) {
            println!("Error: {:?}", e);
            break;
        }
        if i % 5000 == 0 {
            eprintln!("State: {:?}", checker.state());
        }
        i += 1;
    }
}
