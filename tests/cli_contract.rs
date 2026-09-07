#[test]
fn cargo_exposes_manapp_binary() {
    let binary = env!("CARGO_BIN_EXE_manapp");
    assert!(std::path::Path::new(binary).is_file());
}
