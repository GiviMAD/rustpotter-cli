pub fn enable_rustpotter_log() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .with_module_level("rustpotter", log::LevelFilter::Debug)
        .init()
        .unwrap();
}
