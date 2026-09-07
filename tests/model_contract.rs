use manapp::model::{SourceStatus, status_label};

#[test]
fn status_labels_distinguish_available_disabled_and_error() {
    assert_eq!(
        status_label(&SourceStatus::Available {
            executable: "brew".into(),
        }),
        "AVAILABLE"
    );
    assert_eq!(
        status_label(&SourceStatus::Disabled {
            candidates: vec!["brew".into()],
        }),
        "DISABLED"
    );
    assert_eq!(
        status_label(&SourceStatus::Error {
            message: "probe failed".into(),
        }),
        "ERROR"
    );
}
