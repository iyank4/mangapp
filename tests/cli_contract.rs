#[test]
fn cargo_exposes_mangap_binary() {
    let binary = env!("CARGO_BIN_EXE_mangap");
    assert!(std::path::Path::new(binary).is_file());
}
