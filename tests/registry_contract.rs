use manapp::registry::source_registry;

#[test]
fn registry_contains_supported_sources_in_alphabetic_order() {
    let sources = source_registry();
    assert_eq!(sources.len(), 17);
    assert!(
        sources
            .windows(2)
            .all(|pair| { pair[0].name.to_lowercase() <= pair[1].name.to_lowercase() })
    );
    assert!(sources.iter().any(|source| source.name == "Homebrew"));
    assert!(sources.iter().any(|source| source.name == "Mac App Store"));
    assert!(sources.iter().any(|source| source.name == "Composer"));
}

#[test]
fn every_registry_entry_has_complete_metadata() {
    assert!(source_registry().iter().all(|source| {
        !source.id.is_empty()
            && !source.name.is_empty()
            && !source.candidates.is_empty()
            && !source.category.is_empty()
    }));
}
