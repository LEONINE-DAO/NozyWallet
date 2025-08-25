use nozy::notes::{NoteManager, ShieldedNote, NoteType, NoteSelectionStrategy};

#[test]
fn test_note_creation() {
    let mut manager = NoteManager::new();
    
    let note = ShieldedNote::new(
        NoteType::Orchard,
        100000000,
        "u1testaddress".to_string(),
        Some(b"Test memo".to_vec()),
        1000,
        None,
    );
    
    manager.add_note(note.clone());
    
    let retrieved = manager.get_note(&note.id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().value, 100000000);
}

#[test]
fn test_balance_calculation() {
    let mut manager = NoteManager::new();
    
    manager.add_note(ShieldedNote::new(
        NoteType::Orchard,
        100000000,
        "u1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    manager.add_note(ShieldedNote::new(
        NoteType::Sapling,
        200000000,
        "z1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    let total_balance = manager.get_total_balance();
    let orchard_balance = manager.get_balance_by_type(NoteType::Orchard);
    let sapling_balance = manager.get_balance_by_type(NoteType::Sapling);
    
    assert_eq!(total_balance, 300000000);
    assert_eq!(orchard_balance, 100000000);
    assert_eq!(sapling_balance, 200000000);
}

#[test]
fn test_note_selection() {
    let mut manager = NoteManager::new();
    
    manager.add_note(ShieldedNote::new(
        NoteType::Orchard,
        50000000,
        "u1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    manager.add_note(ShieldedNote::new(
        NoteType::Orchard,
        30000000,
        "u1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    manager.add_note(ShieldedNote::new(
        NoteType::Sapling,
        150000000,
        "z1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    let selected = manager.select_notes_for_payment(100000000, NoteSelectionStrategy::PrivacyFirst);
    let total_selected: u64 = selected.iter().map(|n| n.value).sum();
    
    assert!(total_selected >= 100000000);
    assert!(selected.len() >= 2);
}

#[test]
fn test_note_consolidation() {
    let mut manager = NoteManager::new();
    
    manager.add_note(ShieldedNote::new(
        NoteType::Orchard,
        10000000,
        "u1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    manager.add_note(ShieldedNote::new(
        NoteType::Orchard,
        20000000,
        "u1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    manager.add_note(ShieldedNote::new(
        NoteType::Orchard,
        15000000,
        "u1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    let groups = manager.consolidate_notes(50000000);
    assert!(!groups.is_empty());
    
    for group in &groups {
        let group_sum: u64 = group.iter().map(|n| n.value).sum();
        assert!(group_sum <= 50000000);
        assert!(group.len() >= 2);
    }
}

#[test]
fn test_mixed_note_types() {
    let mut manager = NoteManager::new();
    
    manager.add_note(ShieldedNote::new(
        NoteType::Orchard,
        100000000,
        "u1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    manager.add_note(ShieldedNote::new(
        NoteType::Sapling,
        50000000,
        "z1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    let orchard_notes = manager.get_notes_by_type(NoteType::Orchard);
    let sapling_notes = manager.get_notes_by_type(NoteType::Sapling);
    
    assert_eq!(orchard_notes.len(), 1);
    assert_eq!(sapling_notes.len(), 1);
    assert_eq!(orchard_notes[0].value, 100000000);
    assert_eq!(sapling_notes[0].value, 50000000);
}

#[test]
fn test_note_statistics() {
    let mut manager = NoteManager::new();
    
    manager.add_note(ShieldedNote::new(
        NoteType::Orchard,
        100000000,
        "u1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    manager.add_note(ShieldedNote::new(
        NoteType::Sapling,
        50000000,
        "z1testaddress".to_string(),
        None,
        1000,
        None,
    ));
    
    let stats = manager.get_statistics();
    
    assert_eq!(stats.total_balance, 150000000);
    assert_eq!(stats.total_notes, 2);
    assert_eq!(stats.orchard_balance, 100000000);
        assert_eq!(stats.orchard_count, 1);
    assert_eq!(stats.sapling_balance, 50000000);
        assert_eq!(stats.sapling_count, 1);
} 