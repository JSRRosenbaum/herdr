use super::*;
use crate::api::schema::{
    ErrorResponse, Method, Request, ResponseResult, SuccessResponse, TerminalPresentationGetParams,
};

fn presentation_request(
    server: &mut HeadlessServer,
    terminal_id: Option<&str>,
    caller_pane_id: Option<&str>,
) -> String {
    let (respond_to, response_rx) = std::sync::mpsc::channel();
    server.handle_api_request_with_shutdown_check(api::ApiRequestMessage {
        request: Request {
            id: "presentation".into(),
            method: Method::TerminalPresentationGet(TerminalPresentationGetParams {
                terminal_id: terminal_id.map(str::to_owned),
                caller_pane_id: caller_pane_id.map(str::to_owned),
            }),
        },
        respond_to,
        response_write_complete: None,
    });
    response_rx.recv().expect("JSON API response")
}

fn counts(server: &mut HeadlessServer, terminal_id: &str) -> (usize, usize) {
    let response: SuccessResponse =
        serde_json::from_str(&presentation_request(server, Some(terminal_id), None)).unwrap();
    let ResponseResult::TerminalPresentation {
        terminal_id: canonical,
        native_viewers,
        ansi_viewers,
    } = response.result
    else {
        panic!("expected terminal presentation");
    };
    assert_eq!(canonical, terminal_id);
    (native_viewers, ansi_viewers)
}

fn connect_pending_viewer(server: &mut HeadlessServer, client_id: u64) {
    server.handle_server_event(ServerEvent::ClientConnected {
        client_id,
        cols: 100,
        rows: 30,
        cell_width_px: 0,
        cell_height_px: 0,
        pixel_mouse: false,
        writer: crate::server::client_transport::ClientWriter::test_paused(),
    });
}

fn connect_shell(
    server: &mut HeadlessServer,
    client_id: u64,
    tab_id: &str,
) -> (
    std::sync::mpsc::Receiver<Vec<u8>>,
    std::sync::mpsc::Receiver<Vec<u8>>,
) {
    let (writer, control, render) = test_client_writer();
    let mut client = ClientConnection::new_with_mode(
        ClientConnectionMode::ClientShell,
        (80, 24),
        crate::kitty_graphics::HostCellSize::default(),
        client_id,
        RenderEncoding::SemanticFrame,
        Some(writer),
    );
    client.shell_surface_active = true;
    client.shell_location = Some(crate::server::clients::ClientShellLocation::default());
    server.clients.insert(client_id, client);
    assert!(server.focus_shell_client_on_tab(client_id, tab_id));
    (control, render)
}

#[test]
fn presentation_api_counts_native_observer_and_controller_lifecycle() {
    with_terminal_session_test_server(|server, terminal, terminal_id, pane_id| {
        server.app.state.active = Some(0);
        assert_eq!(counts(server, &terminal_id), (0, 0));
        connect_pending_viewer(server, 7);
        assert_eq!(counts(server, &terminal_id), (0, 0));
        assert!(
            server.handle_server_event(ServerEvent::ClientNativeAttachTerminal {
                client_id: 7,
                terminal_id: terminal_id.clone(),
            })
        );
        assert_eq!(counts(server, &terminal_id), (1, 0));
        assert_eq!(server.terminal_attach_owners.get(&terminal_id), Some(&7));

        connect_pending_viewer(server, 8);
        assert!(
            server.handle_server_event(ServerEvent::ClientObserveTerminal {
                client_id: 8,
                target: terminal_id.clone(),
            })
        );
        assert_eq!(counts(server, &terminal_id), (1, 1));
        let runtime = server.app.terminal_runtimes.remove(&terminal).unwrap();
        assert_eq!(counts(server, &terminal_id), (0, 1));
        server
            .app
            .terminal_runtimes
            .insert(terminal.clone(), runtime);
        assert_eq!(server.terminal_attach_owners.get(&terminal_id), Some(&7));
        server.handle_server_event(ServerEvent::ClientDisconnected { client_id: 8 });
        assert_eq!(counts(server, &terminal_id), (1, 0));

        let resolved: SuccessResponse =
            serde_json::from_str(&presentation_request(server, None, Some(&pane_id))).unwrap();
        assert!(
            matches!(resolved.result, ResponseResult::TerminalPresentation {
            terminal_id: canonical, native_viewers: 1, ansi_viewers: 0
        } if canonical == terminal_id)
        );

        connect_pending_viewer(server, 9);
        assert!(
            !server.handle_server_event(ServerEvent::ClientAttachTerminal {
                client_id: 9,
                terminal_id: terminal_id.clone(),
                takeover: false,
            })
        );
        assert_eq!(counts(server, &terminal_id), (1, 0));
        connect_pending_viewer(server, 10);
        assert!(
            server.handle_server_event(ServerEvent::ClientAttachTerminal {
                client_id: 10,
                terminal_id: terminal_id.clone(),
                takeover: true,
            })
        );
        assert_eq!(counts(server, &terminal_id), (0, 1));
        assert_eq!(server.terminal_attach_owners.get(&terminal_id), Some(&10));
        server.handle_server_event(ServerEvent::ClientDisconnected { client_id: 10 });
        assert_eq!(counts(server, &terminal_id), (0, 0));
    });
}

#[test]
fn presentation_api_excludes_closed_writers_before_disconnect() {
    with_terminal_session_test_server(|server, _, terminal_id, _| {
        server.app.state.active = Some(0);
        let tab_id = server.app.public_tab_id(0, 0).unwrap();
        connect_pending_viewer(server, 7);
        assert!(
            server.handle_server_event(ServerEvent::ClientNativeAttachTerminal {
                client_id: 7,
                terminal_id: terminal_id.clone(),
            })
        );
        assert_eq!(counts(server, &terminal_id), (1, 0));
        assert!(server.clients.get(&7).unwrap().native_output.is_some());
        server
            .clients
            .get(&7)
            .unwrap()
            .writer
            .as_ref()
            .unwrap()
            .test_close();
        assert!(server.clients.contains_key(&7));
        assert_eq!(counts(server, &terminal_id), (0, 0));

        connect_pending_viewer(server, 8);
        assert!(
            server.handle_server_event(ServerEvent::ClientObserveTerminal {
                client_id: 8,
                target: terminal_id.clone(),
            })
        );
        assert_eq!(counts(server, &terminal_id), (0, 1));
        server
            .clients
            .get(&8)
            .unwrap()
            .writer
            .as_ref()
            .unwrap()
            .test_close();
        assert_eq!(counts(server, &terminal_id), (0, 0));

        connect_pending_viewer(server, 9);
        assert!(
            server.handle_server_event(ServerEvent::ClientAttachTerminal {
                client_id: 9,
                terminal_id: terminal_id.clone(),
                takeover: true,
            })
        );
        assert_eq!(counts(server, &terminal_id), (0, 1));
        server
            .clients
            .get(&9)
            .unwrap()
            .writer
            .as_ref()
            .unwrap()
            .test_close();
        assert_eq!(counts(server, &terminal_id), (0, 0));

        let _shell_output = connect_shell(server, 20, &tab_id);
        assert_eq!(counts(server, &terminal_id), (0, 1));
        server
            .clients
            .get(&20)
            .unwrap()
            .writer
            .as_ref()
            .unwrap()
            .test_close();
        assert_eq!(counts(server, &terminal_id), (0, 0));

        connect_pending_viewer(server, 10);
        assert!(
            server.handle_server_event(ServerEvent::ClientObserveTerminal {
                client_id: 10,
                target: terminal_id.clone(),
            })
        );
        assert_eq!(counts(server, &terminal_id), (0, 1));
    });
}

#[test]
fn presentation_api_tracks_client_local_tab_zoom_surface_and_popup_visibility() {
    with_terminal_session_test_server(|server, terminal, terminal_id, _| {
        server.app.state.active = Some(0);
        let second_tab = server.app.state.workspaces[0].test_add_tab(Some("other"));
        server.app.state.ensure_test_terminals();
        let first_tab_id = server.app.public_tab_id(0, 0).unwrap();
        let second_tab_id = server.app.public_tab_id(0, second_tab).unwrap();
        let second_terminal_id = server.app.state.workspaces[0].tabs[second_tab]
            .terminal_id(server.app.state.workspaces[0].tabs[second_tab].root_pane)
            .unwrap()
            .to_string();
        connect_pending_viewer(server, 7);
        assert!(
            server.handle_server_event(ServerEvent::ClientNativeAttachTerminal {
                client_id: 7,
                terminal_id: terminal_id.clone(),
            })
        );
        let _shell_output = connect_shell(server, 20, &second_tab_id);
        assert_eq!(counts(server, &terminal_id), (1, 0));
        connect_pending_viewer(server, 8);
        assert!(
            server.handle_server_event(ServerEvent::ClientObserveTerminal {
                client_id: 8,
                target: second_terminal_id,
            })
        );
        assert_eq!(counts(server, &terminal_id), (1, 0));
        assert!(server.focus_shell_client_on_tab(20, &first_tab_id));
        assert_eq!(counts(server, &terminal_id), (1, 1));
        assert!(server.set_client_shell_surface_active(20, false).is_some());
        assert_eq!(counts(server, &terminal_id), (1, 0));
        assert!(server.set_client_shell_surface_active(20, true).is_some());

        let workspace = &mut server.app.state.workspaces[0];
        let other_pane = workspace.test_split(ratatui::layout::Direction::Horizontal);
        workspace.tabs[0].zoomed = true;
        assert_ne!(
            workspace.tabs[0].layout.focused(),
            workspace.tabs[0].root_pane
        );
        assert_ne!(workspace.terminal_id(other_pane), Some(&terminal));
        server.app.state.ensure_test_terminals();
        assert_eq!(counts(server, &terminal_id), (1, 0));
        server.app.state.workspaces[0].tabs[0].zoomed = false;
        assert_eq!(counts(server, &terminal_id), (1, 1));
        assert!(server.focus_shell_client_on_tab(20, &second_tab_id));
        assert_eq!(counts(server, &terminal_id), (1, 0));

        let popup_runtime = crate::terminal::TerminalRuntime::test_with_screen_bytes(40, 12, b"");
        let (_, popup_terminal_id) = server.app.install_test_popup_runtime(popup_runtime);
        server.popup_owner_tab_id = Some(first_tab_id.clone());
        assert_eq!(counts(server, popup_terminal_id.as_str()), (0, 0));
        assert!(server.focus_shell_client_on_tab(20, &first_tab_id));
        assert_eq!(counts(server, popup_terminal_id.as_str()), (0, 1));
        server.app.close_popup_pane();
    });
}

#[test]
fn presentation_api_rejects_missing_and_ambiguous_targets_without_admitting_viewers() {
    with_terminal_session_test_server(|server, terminal, terminal_id, pane_id| {
        server.app.state.active = Some(0);
        server.app.state.public_pane_id_aliases.insert(
            "old:p1".into(),
            server.app.state.workspaces[0].tabs[0].root_pane,
        );
        let response: SuccessResponse =
            serde_json::from_str(&presentation_request(server, None, Some("old:p1"))).unwrap();
        assert!(
            matches!(response.result, ResponseResult::TerminalPresentation {
            terminal_id: canonical, native_viewers: 0, ansi_viewers: 0
        } if canonical == terminal_id)
        );
        for (target, caller, code) in [
            (None, None, "invalid_params"),
            (
                Some(terminal_id.as_str()),
                Some(pane_id.as_str()),
                "invalid_params",
            ),
            (Some("term_missing"), None, "terminal_not_found"),
            (None, Some("missing:p1"), "terminal_not_found"),
        ] {
            let error: ErrorResponse =
                serde_json::from_str(&presentation_request(server, target, caller)).unwrap();
            assert_eq!(error.error.code, code);
        }
        connect_pending_viewer(server, 31);
        assert_eq!(counts(server, &terminal_id), (0, 0));
        assert!(
            !server.handle_server_event(ServerEvent::ClientNativeAttachTerminal {
                client_id: 31,
                terminal_id: "term_missing".into(),
            })
        );
        assert_eq!(counts(server, &terminal_id), (0, 0));
        let runtime = server.app.terminal_runtimes.remove(&terminal).unwrap();
        connect_pending_viewer(server, 32);
        assert!(
            !server.handle_server_event(ServerEvent::ClientNativeAttachTerminal {
                client_id: 32,
                terminal_id: terminal_id.clone(),
            })
        );
        assert!(!server.clients.contains_key(&32));
        assert_eq!(counts(server, &terminal_id), (0, 0));
        server
            .app
            .terminal_runtimes
            .insert(terminal.clone(), runtime);
        assert!(server.app.state.terminals.contains_key(&terminal));
    });
}
