use super::*;
use crate::api::schema::{
    ErrorBody, ErrorResponse, ResponseResult, SuccessResponse, TerminalPresentationGetParams,
};

impl HeadlessServer {
    pub(super) fn terminal_presentation_response(
        &self,
        request_id: &str,
        params: &TerminalPresentationGetParams,
    ) -> String {
        let terminal_id = match (&params.terminal_id, &params.caller_pane_id) {
            (Some(terminal_id), None) => self
                .app
                .state
                .terminals
                .get_key_value(terminal_id.as_str())
                .map(|(id, _)| id),
            (None, Some(caller_pane_id)) => self
                .app
                .parse_pane_id(caller_pane_id)
                .and_then(|(ws_idx, pane_id)| {
                    self.app.state.workspaces.get(ws_idx)?.terminal_id(pane_id)
                })
                .filter(|id| self.app.state.terminals.contains_key(*id)),
            _ => {
                return presentation_error(
                    request_id,
                    "invalid_params",
                    "exactly one of terminal_id or caller_pane_id is required",
                );
            }
        };
        let Some(terminal_id) = terminal_id else {
            return presentation_error(request_id, "terminal_not_found", "terminal not found");
        };
        let has_runtime = self.app.terminal_runtimes.get(terminal_id).is_some();
        let terminal_id = terminal_id.as_str();
        let mut native_viewers = 0;
        let mut ansi_viewers = 0;
        for (&client_id, client) in &self.clients {
            // Pending admission and closed writers are not viewers, even before disconnect.
            if !client
                .writer
                .as_ref()
                .is_some_and(|writer| writer.is_alive())
            {
                continue;
            }
            match &client.mode {
                ClientConnectionMode::TerminalAttach {
                    terminal_id: attached,
                } if attached == terminal_id => {
                    if client
                        .native_output
                        .as_ref()
                        .is_some_and(|output| !output.is_closed())
                        && has_runtime
                    {
                        native_viewers += 1;
                    } else if !client.native_requesting {
                        ansi_viewers += 1;
                    }
                }
                ClientConnectionMode::TerminalObserve {
                    terminal_id: observed,
                } if observed == terminal_id => {
                    ansi_viewers += 1;
                }
                ClientConnectionMode::ClientShell if client.shell_surface_active => {
                    let visible_tab = self.shell_target_for_client(client_id).and_then(|target| {
                        self.app
                            .state
                            .workspaces
                            .get(target.workspace_index)?
                            .tabs
                            .get(target.tab_index)
                            .map(|tab| (target, tab))
                    });
                    let shows_tab_pane = visible_tab.is_some_and(|(_, tab)| {
                        if tab.zoomed {
                            tab.terminal_id(tab.layout.focused())
                                .is_some_and(|id| id.as_str() == terminal_id)
                        } else {
                            tab.panes
                                .values()
                                .any(|pane| pane.attached_terminal_id.as_str() == terminal_id)
                        }
                    });
                    let shows_popup = !shows_tab_pane
                        && visible_tab.is_some_and(|(target, _)| {
                            self.app
                                .state
                                .popup_pane
                                .as_ref()
                                .is_some_and(|popup| popup.terminal_id.as_str() == terminal_id)
                                && self
                                    .popup_owner_tab_id
                                    .as_deref()
                                    .and_then(|tab_id| self.app.parse_tab_id(tab_id))
                                    == Some((target.workspace_index, target.tab_index))
                        });
                    if shows_tab_pane || shows_popup {
                        ansi_viewers += 1;
                    }
                }
                _ => {}
            }
        }
        serde_json::to_string(&SuccessResponse {
            id: request_id.to_owned(),
            result: ResponseResult::TerminalPresentation {
                terminal_id: terminal_id.to_owned(),
                native_viewers,
                ansi_viewers,
            },
        })
        .unwrap_or_else(|err| {
            presentation_error(request_id, "serialization_error", &err.to_string())
        })
    }
}

fn presentation_error(id: &str, code: &str, message: &str) -> String {
    serde_json::to_string(&ErrorResponse {
        id: id.to_owned(),
        error: ErrorBody {
            code: code.to_owned(),
            message: message.to_owned(),
        },
    })
    .unwrap_or_else(|_| {
        r#"{"id":"","error":{"code":"serialization_error","message":"response encoding failed"}}"#
            .to_owned()
    })
}
