p = "src/client/shell/tests/agents_worktrees_notifications.rs"
s = open(p).read()
old = "fn mouse_hits_use_stable_workspace_tab_and_pane_ids() {"
dot = chr(0xB7)
capture = (
    "fn capture_compact_layout_modes() {\n"
    "    let snapshot = compact_grouped_snapshot();\n"
    "    let mut out = String::new();\n"
    "    for mode in [(WorktreeLayout::Tree, \"tree\"), (WorktreeLayout::Compact, \"compact\")] {\n"
    "        let (layout, name) = mode;\n"
    "        let mut config = ClientShellConfig::from_config(&Config::default());\n"
    "        config.spaces.worktree_layout = layout;\n"
    "        let mut state = ClientShellState::new(config);\n"
    "        state.set_snapshot(Box::new(snapshot.clone()));\n"
    "        state.set_pane_surface(surface());\n"
    "        let frame = state.compose(106, 20).expect(\"frame\");\n"
    "        out.push_str(&format!(\"\\n=== worktree_layout = {name} ===\\n\"));\n"
    "        for row in frame.cells.chunks(frame.width as usize) {\n"
    "            let line: String = row\n"
    "                .iter()\n"
    "                .map(|cell| cell.symbol.as_str())\n"
    "                .collect::<String>();\n"
    "            out.push_str(&line.replace(' ', \"%DOT%\"));\n"
    "            out.push('\\n');\n"
    "        }\n"
    "    }\n"
    "    std::fs::write(\"/tmp/sidebar-captures/compact.txt\", &out).expect(\"write capture\");\n"
    "}\n"
    "\n"
)
capture = capture.replace("%DOT%", dot)
assert s.count(old) == 1
open(p, "w").write(s.replace(old, capture + old, 1))
print("inserted")
