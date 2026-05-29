use ghostty_vt::{test_allocator, ActionTag, Notification, NotificationTag, Viewer, ViewerState};

fn notification(tag: NotificationTag) -> Notification {
    Notification {
        tag,
        pane_id: 0,
        data_ptr: core::ptr::null(),
        data_len: 0,
        id: 0,
        name_ptr: core::ptr::null(),
        name_len: 0,
        window_id: 0,
        layout_ptr: core::ptr::null(),
        layout_len: 0,
        visible_layout_ptr: core::ptr::null(),
        visible_layout_len: 0,
        raw_flags_ptr: core::ptr::null(),
        raw_flags_len: 0,
        client_ptr: core::ptr::null(),
        client_len: 0,
        session_id: 0,
    }
}

fn notification_with_data(tag: NotificationTag, data: &[u8]) -> Notification {
    Notification {
        tag,
        data_ptr: data.as_ptr(),
        data_len: data.len(),
        ..notification(tag)
    }
}

fn notification_session(id: usize) -> Notification {
    Notification {
        tag: NotificationTag::SessionChanged,
        id,
        ..notification(NotificationTag::SessionChanged)
    }
}

fn notification_output(pane_id: usize, data: &[u8]) -> Notification {
    Notification {
        tag: NotificationTag::Output,
        pane_id,
        data_ptr: data.as_ptr(),
        data_len: data.len(),
        ..notification(NotificationTag::Output)
    }
}

fn actions_slice(viewer: &mut Viewer, n: Notification) -> Vec<(ActionTag, String)> {
    let (ptr, len) = viewer.next(&n);
    if ptr.is_null() || len == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let action = unsafe { &*ptr.add(i) };
        let tag = action.tag;
        let cmd = if action.command_len > 0 && !action.command_ptr.is_null() {
            let bytes =
                unsafe { core::slice::from_raw_parts(action.command_ptr, action.command_len) };
            String::from_utf8_lossy(bytes).into_owned()
        } else {
            String::new()
        };
        out.push((tag, cmd));
    }
    out
}

fn has_command(actions: &[(ActionTag, String)], needle: &str) -> bool {
    actions
        .iter()
        .any(|(tag, cmd)| *tag == ActionTag::Command && cmd.contains(needle))
}

#[test]
fn immediate_exit() {
    let alloc = test_allocator();
    let mut viewer = Viewer::init(&alloc);
    let actions = actions_slice(&mut viewer, notification(NotificationTag::Exit));
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].0, ActionTag::Exit);

    let (ptr, len) = viewer.next(&notification(NotificationTag::Exit));
    assert!(ptr.is_null() || len == 0);
}

#[test]
fn initial_flow_list_windows_and_panes() {
    let alloc = test_allocator();
    let mut viewer = Viewer::init(&alloc);

    let _ = actions_slice(
        &mut viewer,
        notification_with_data(NotificationTag::BlockEnd, b""),
    );
    let actions = actions_slice(&mut viewer, notification_session(42));
    assert!(has_command(&actions, "display-message"));
    assert_eq!(viewer.session_id, 42);

    let actions = actions_slice(
        &mut viewer,
        notification_with_data(NotificationTag::BlockEnd, b"3.5a"),
    );
    assert!(has_command(&actions, "list-windows"));
    assert_eq!(viewer.tmux_version_bytes(), b"3.5a");

    let list_windows = b"$0 @0 83 44 027b,83x44,0,0[83x20,0,0,0,83x23,0,21,1]\n";
    let actions = actions_slice(
        &mut viewer,
        notification_with_data(NotificationTag::BlockEnd, list_windows),
    );
    assert!(actions.iter().any(|(tag, _)| *tag == ActionTag::Windows));
    assert!(has_command(&actions, "capture-pane"));
    assert_eq!(viewer.window_count(), 1);
    assert_eq!(viewer.pane_count(), 2);
    assert!(viewer.pane_has_terminal(0));
    assert!(viewer.pane_has_terminal(1));
}

#[test]
fn pane_capture_visible_then_live_output() {
    let alloc = test_allocator();
    let mut viewer = Viewer::init(&alloc);

    let _ = actions_slice(
        &mut viewer,
        notification_with_data(NotificationTag::BlockEnd, b""),
    );
    let _ = actions_slice(&mut viewer, notification_session(0));
    let _ = actions_slice(
        &mut viewer,
        notification_with_data(NotificationTag::BlockEnd, b"3.5a"),
    );
    let _ = actions_slice(
        &mut viewer,
        notification_with_data(
            NotificationTag::BlockEnd,
            b"$0 @0 83 44 027b,83x44,0,0[83x20,0,0,0,83x23,0,21,1]\n",
        ),
    );

    // pane 0 history
    let _ = actions_slice(
        &mut viewer,
        notification_with_data(NotificationTag::BlockEnd, b"Hello, world!\n"),
    );
    // pane 0 visible
    let actions = actions_slice(
        &mut viewer,
        notification_with_data(NotificationTag::BlockEnd, b"Hello, world!\n"),
    );
    assert!(has_command(&actions, "capture-pane"));
    assert!(viewer.pane_has_terminal(0));

    // drain remaining capture commands for both panes / alternate screens
    for _ in 0..6 {
        let _ = actions_slice(
            &mut viewer,
            notification_with_data(NotificationTag::BlockEnd, b""),
        );
    }

    let actions = actions_slice(&mut viewer, notification_output(0, b"new output"));
    assert!(actions.is_empty());
    assert!(viewer.pane_has_terminal(0));

    let actions = actions_slice(&mut viewer, notification_output(999, b"ignored"));
    assert!(actions.is_empty());
}

#[test]
fn session_changed_resets_panes() {
    let alloc = test_allocator();
    let mut viewer = Viewer::init(&alloc);

    let _ = actions_slice(
        &mut viewer,
        notification_with_data(NotificationTag::BlockEnd, b""),
    );
    let _ = actions_slice(&mut viewer, notification_session(1));
    let _ = actions_slice(
        &mut viewer,
        notification_with_data(NotificationTag::BlockEnd, b"3.5a"),
    );
    let _ = actions_slice(
        &mut viewer,
        notification_with_data(
            NotificationTag::BlockEnd,
            b"$1 @0 83 44 027b,83x44,0,0[83x20,0,0,0,83x23,0,21,1]\n",
        ),
    );
    assert_eq!(viewer.pane_count(), 2);

    let actions = actions_slice(&mut viewer, notification_session(2));
    assert!(actions.iter().any(|(tag, _)| *tag == ActionTag::Windows));
    assert!(has_command(&actions, "list-windows"));
    assert_eq!(viewer.session_id, 2);
    assert_eq!(viewer.window_count(), 0);
    assert_eq!(viewer.pane_count(), 0);
    assert_eq!(viewer.tmux_version_bytes(), b"3.5a");

    let actions = actions_slice(
        &mut viewer,
        notification_with_data(
            NotificationTag::BlockEnd,
            b"$2 @1 83 44 027b,83x44,0,0[83x20,0,0,0,83x23,0,21,1]\n",
        ),
    );
    assert_eq!(viewer.pane_count(), 2);
    assert!(has_command(&actions, "capture-pane"));
    assert!(matches!(viewer.state, ViewerState::CommandQueue));
}
