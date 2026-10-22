# Windows saved-SSH responsiveness: recorded timing evidence

These are historical local measurements of this fix, not guarantees or measurements of an official upstream release. Portable endpoint labels omit machine names, addresses, profile IDs, sessions, and user paths.

The measurement source was the fork baseline `c9cdaa124b6e7cee03010a4fdb20c9cddd1a5211` with staged local changes. The archived full local patch has SHA-256 `fc574f02d9df2dfa3346beaaa3487c1b3a1e4e878f11cedb6fbacde4a1051396`. This PR ports its source changes onto the newer fork base; it does not claim that the exact current PR head was benchmarked.

## Saved-SSH welcome/handshake time

Both arms already included the transport changes; this pair isolates the saved-executable metadata startup change. The candidate reuses valid saved metadata. Each endpoint has one before observation and one after observation from the same startup probe, using the probe's handshake duration rather than total test-process wall time. Cache, server load, and network conditions were not normalized.

| Endpoint | Before | After | Time saved | Reduction | Speedup |
|---|---:|---:|---:|---:|---:|
| A | 7.800 s | 2.364 s | 5.436 s | 69.7% | 3.3x |
| B | 9.895 s | 2.762 s | 7.133 s | 72.1% | 3.6x |

This measures endpoint welcome/handshake, not first visible desktop frame. There is no paired baseline for the other saved endpoints, so no delta is claimed for them.

## Native pipe buffer-capacity A/B

A standalone Windows reproduction used the locked `interprocess 2.4.4` dependency and dedicated local pipes. Each condition transferred and checked 20 payloads of 52 KiB (1,064,960 bytes total), using nonblocking 512-byte writes, a 1 ms writer wait, and a 2 ms peek-before-read wait. The varied parameter was pipe buffer capacity; this is not an application-version or network benchmark. There was one run per condition, with ordinary desktop background activity and no CPU/cache normalization.

| Metric | 512-byte buffer | 64 KiB buffer | Observed change |
|---|---:|---:|---:|
| Transfer elapsed time | 4.403 s | 0.039 s | 4.364 s less; approximately 113x faster |

The speedup is approximate because the recorded elapsed values are rounded. The older dependency-version reproduction is not used in this comparison.

## Coherent pane views after the fix

These measurements used the actual Windows SSH bridge and native endpoint transport against isolated remote servers with generated scrollback. They verified matching snapshot/frame revisions, expected pane text, and scroll position—not just request acknowledgements. Geometry was 153 columns by 70 rows. Each endpoint had 20 workspace-focus samples and eight scroll samples.

| Endpoint | Focus median / maximum | Scroll median / maximum |
|---|---:|---:|
| A | 18.1 / 23.2 ms | 18.4 / 22.4 ms |
| B | 31.7 / 54.8 ms | 41.0 / 58.7 ms |

No pre-fix coherent-view series was recorded. The user's earlier 30–45-second workspace-switch stalls were observations from real workspaces, not the same isolated fixture. Comparing those observations directly with this table would not establish a measured speedup.

## Correctness evidence and limitations

- Historical focused verification passed 33 native-Windows checks and 119 Unix checks. The temporary manual probe was removed afterward.
- The standalone dirty-pipe reproduction completed 200 same-name reconnect cycles after explicit abort; the unpatched dirty-drop case returned access denied.
- No desktop GUI automation or live-user workspace changes were used in the isolated view measurements.
- The historical effective Rust compiler version was not recorded. A requested toolchain environment variable must not be treated as proof of the selected compiler.
- The saved traces are supporting evidence with small sample counts, not CI performance thresholds. Current-branch CI and regression results should be read separately from these historical timings.
