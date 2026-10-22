/// Optional same-install named controls; never change the frozen endpoint codec.
pub(crate) const NATIVE_ATTACH_KIND: &str = "terminal.native.attach.v1";
pub(crate) const NATIVE_READY_KIND: &str = "terminal.native.ready.v1";
pub(crate) const NATIVE_BYTES_KIND: &str = "terminal.native.bytes.v1";

pub(crate) fn paste_payload_for_runtime(
    runtime: &crate::terminal::TerminalRuntime,
    text: &str,
) -> String {
    if runtime.bracketed_paste_enabled() {
        format!("\x1b[200~{text}\x1b[201~")
    } else {
        text.to_owned()
    }
}
