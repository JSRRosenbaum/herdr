use super::*;

/// Runs the thin client and enters the main event loop.
pub fn run_client() -> io::Result<()> {
    run_client_with_mode(None, None, "connecting to server", None, false)
}

#[cfg(unix)]
pub fn run_terminal_attach(terminal_id: String, takeover: bool) -> io::Result<()> {
    run_terminal_attach_on_socket(None, terminal_id, takeover)
}

#[cfg(unix)]
pub(crate) fn run_terminal_attach_on_socket(
    socket: Option<&std::path::Path>,
    terminal_id: String,
    takeover: bool,
) -> io::Result<()> {
    run_client_with_mode(
        Some((terminal_id, takeover)),
        Some(AttachEscapeState::default()),
        "attaching to terminal",
        socket,
        false,
    )
}

#[cfg(unix)]
pub fn run_native_terminal_attach(terminal_id: String) -> io::Result<()> {
    run_native_terminal_attach_on_socket(None, terminal_id)
}

#[cfg(unix)]
pub(crate) fn run_native_terminal_attach_on_socket(
    socket: Option<&std::path::Path>,
    terminal_id: String,
) -> io::Result<()> {
    run_client_with_mode(
        Some((terminal_id, false)),
        Some(AttachEscapeState::default()),
        "attaching native terminal",
        socket,
        true,
    )
}

#[cfg(windows)]
pub fn run_native_terminal_attach(_terminal_id: String) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "native terminal attach requires a Unix client; use a saved SSH profile from Tern",
    ))
}

#[cfg(windows)]
pub fn run_terminal_attach(_terminal_id: String, _takeover: bool) -> io::Result<()> {
    debug_assert!(!crate::platform::capabilities().direct_terminal_attach);
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "direct terminal attach is not supported on Windows yet",
    ))
}
