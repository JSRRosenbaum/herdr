#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Position {
    Cell { column: u16, row: u16 },
    Pixels { x: u32, y: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HostGeometry {
    pub(crate) cols: u16,
    pub(crate) rows: u16,
    pub(crate) width_px: u32,
    pub(crate) height_px: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HostPixels {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) geometry: HostGeometry,
}

impl HostGeometry {
    pub(crate) fn new(cols: u16, rows: u16, width_px: u32, height_px: u32) -> Option<Self> {
        (cols > 0 && rows > 0 && width_px > 0 && height_px > 0).then_some(Self {
            cols,
            rows,
            width_px,
            height_px,
        })
    }

    #[cfg(unix)]
    pub(crate) fn current(reported_cell_size: Option<(u32, u32)>) -> Option<Self> {
        Self::from_window_size(
            &crossterm::terminal::window_size().ok()?,
            reported_cell_size,
        )
    }

    #[cfg(any(unix, test))]
    fn from_window_size(
        size: &crossterm::terminal::WindowSize,
        reported_cell_size: Option<(u32, u32)>,
    ) -> Option<Self> {
        let (width_px, height_px) = match reported_cell_size {
            Some((width, height)) => (
                u32::from(size.columns).checked_mul(width)?,
                u32::from(size.rows).checked_mul(height)?,
            ),
            None => (u32::from(size.width), u32::from(size.height)),
        };
        Self::new(size.columns, size.rows, width_px, height_px)
    }

    pub(crate) fn cell(self, x: u32, y: u32) -> Option<(u16, u16)> {
        let x = x.checked_sub(1)?;
        let y = y.checked_sub(1)?;
        if x >= self.width_px || y >= self.height_px {
            return None;
        }
        let width_px = grid_extent(self.cols, self.width_px)?;
        let height_px = grid_extent(self.rows, self.height_px)?;
        Some((
            grid_cell(x.min(width_px - 1), self.cols, width_px)?,
            grid_cell(y.min(height_px - 1), self.rows, height_px)?,
        ))
    }
}

impl HostPixels {
    pub(crate) fn pane_position(
        self,
        inner: ratatui::layout::Rect,
        child_width_px: u32,
        child_height_px: u32,
    ) -> Option<Position> {
        let (host_column, host_row) = self.geometry.cell(self.x, self.y)?;
        let end_column = inner.x.checked_add(inner.width)?;
        let end_row = inner.y.checked_add(inner.height)?;
        if host_column < inner.x
            || host_column >= end_column
            || host_row < inner.y
            || host_row >= end_row
        {
            return None;
        }
        Some(Position::Pixels {
            x: map_axis_within_cell(
                self.x,
                host_column,
                inner.x,
                inner.width,
                self.geometry.cols,
                self.geometry.width_px,
                child_width_px,
            )?,
            y: map_axis_within_cell(
                self.y,
                host_row,
                inner.y,
                inner.height,
                self.geometry.rows,
                self.geometry.height_px,
                child_height_px,
            )?,
        })
    }
}

fn map_axis_within_cell(
    pixel: u32,
    host_cell: u16,
    pane_start: u16,
    pane_cells: u16,
    host_cells: u16,
    host_extent: u32,
    child_extent: u32,
) -> Option<u32> {
    let local_cell = host_cell.checked_sub(pane_start)?;
    if local_cell >= pane_cells {
        return None;
    }
    let host_grid_extent = grid_extent(host_cells, host_extent)?;
    let source_start = boundary(host_cell, host_cells, host_grid_extent)?;
    let source_end = boundary(host_cell.checked_add(1)?, host_cells, host_grid_extent)?;
    let target_start = boundary(local_cell, pane_cells, child_extent)?;
    let target_end = boundary(local_cell.checked_add(1)?, pane_cells, child_extent)?;
    let source_width = source_end.checked_sub(source_start)?;
    let target_width = target_end.checked_sub(target_start)?;
    let offset = pixel.checked_sub(1)?.checked_sub(source_start)?;
    if source_width == 0 || target_width == 0 || offset >= source_width {
        return None;
    }
    target_start
        .checked_add(scale(offset, source_width, target_width))?
        .checked_add(1)
}

fn boundary(index: u16, count: u16, extent: u32) -> Option<u32> {
    (count > 0 && index <= count && extent > 0)
        .then(|| (u64::from(index) * u64::from(extent) / u64::from(count)) as u32)
}

fn grid_extent(count: u16, extent: u32) -> Option<u32> {
    let count = u32::from(count);
    (count > 0 && extent > 0).then(|| (extent / count).max(1) * count)
}

fn grid_cell(pixel: u32, count: u16, extent: u32) -> Option<u16> {
    if count == 0 || extent == 0 || pixel >= extent {
        return None;
    }
    let cell = ((u64::from(pixel) + 1) * u64::from(count) - 1) / u64::from(extent);
    u16::try_from(cell).ok().filter(|cell| *cell < count)
}

/// Side-button SGR reports cannot round-trip through crossterm's mouse buttons.
#[cfg(any(unix, test))]
pub(crate) fn navigation_report(data: &[u8]) -> Option<(u32, u32, u32, u8)> {
    let body = data.strip_prefix(b"\x1b[<")?;
    let (&suffix, fields) = body.split_last()?;
    if !matches!(suffix, b'M' | b'm') {
        return None;
    }
    let parse_number = |field: &[u8]| {
        (!field.is_empty() && field.iter().all(u8::is_ascii_digit))
            .then(|| std::str::from_utf8(field).ok()?.parse::<u32>().ok())
            .flatten()
    };
    let mut fields = fields.split(|byte| *byte == b';');
    let button = parse_number(fields.next()?)?;
    if !matches!(button & !28, 128 | 129) {
        return None;
    }
    let x = parse_number(fields.next()?)?;
    let y = parse_number(fields.next()?)?;
    fields.next().is_none().then_some((button, x, y, suffix))
}

fn scale(pixel: u32, source: u32, target: u32) -> u32 {
    ((u64::from(pixel) * u64::from(target)) / u64::from(source))
        .min(u64::from(target.saturating_sub(1))) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_reports_preserve_only_side_presses_and_releases() {
        for button in [128, 129] {
            for modifiers in 0..8 {
                let code = button | (modifiers << 2);
                for suffix in [b'M', b'm'] {
                    let report = format!("\x1b[<{code};12;34{}", suffix as char);
                    assert_eq!(
                        navigation_report(report.as_bytes()),
                        Some((code, 12, 34, suffix))
                    );
                }
            }
        }
        for code in [0, 1, 2, 64, 65, 130, 131, 160, 161, 192] {
            let report = format!("\x1b[<{code};12;34M");
            assert_eq!(navigation_report(report.as_bytes()), None);
        }
        for report in [
            b"\x1b[<128;12;34".as_slice(),
            b"\x1b[<128;12;34Mtext",
            b"\x1b[<128;12;34;56M",
            b"\x1b[<128;12;badM",
            b"text",
        ] {
            assert_eq!(navigation_report(report), None);
        }
    }

    #[test]
    fn reported_cell_pitch_corrects_ssh_pixel_hit_testing() {
        let mut size = crossterm::terminal::WindowSize {
            columns: 251,
            rows: 86,
            width: 640,
            height: 480,
        };
        let unconfirmed = HostGeometry::from_window_size(&size, None).unwrap();
        assert_eq!(unconfirmed.cell(1001, 441), None);

        let reported = HostGeometry::from_window_size(&size, Some((10, 22))).unwrap();
        assert_eq!(reported.cell(1001, 441), Some((100, 20)));
        assert_eq!(reported.cell(2510, 1892), Some((250, 85)));
        assert_eq!(reported.cell(2511, 1892), None);
        let pane = ratatui::layout::Rect::new(27, 0, 224, 85);
        assert_eq!(
            HostPixels {
                x: 1001,
                y: 441,
                geometry: reported,
            }
            .pane_position(pane, 2240, 1870),
            Some(Position::Pixels { x: 731, y: 441 })
        );

        size.width = 0;
        size.height = 0;
        assert_eq!(
            HostGeometry::from_window_size(&size, Some((10, 22)))
                .unwrap()
                .cell(1001, 441),
            Some((100, 20))
        );
    }

    #[test]
    fn integer_cell_pitch_ignores_trailing_pixel_remainder() {
        let geometry = HostGeometry::new(127, 31, 1_276, 626).unwrap();
        assert_eq!(geometry.cell(220, 1), Some((21, 0)));
        assert_eq!(geometry.cell(221, 61), Some((22, 3)));
        let right_pane = ratatui::layout::Rect::new(22, 3, 105, 20);
        assert_eq!(
            HostPixels {
                x: 221,
                y: 61,
                geometry,
            }
            .pane_position(right_pane, 1_050, 400),
            Some(Position::Pixels { x: 1, y: 1 })
        );
        assert_eq!(geometry.cell(1_270, 620), Some((126, 30)));
        assert_eq!(
            HostPixels {
                x: 1_270,
                y: 460,
                geometry,
            }
            .pane_position(right_pane, 1_050, 400),
            Some(Position::Pixels { x: 1_050, y: 400 })
        );

        let full_grid = ratatui::layout::Rect::new(0, 0, 127, 31);
        for x in 1_271..=1_276 {
            assert_eq!(geometry.cell(x, 1), Some((126, 0)));
            assert_eq!(
                HostPixels { x, y: 1, geometry }.pane_position(full_grid, 1_270, 620),
                None
            );
        }
        for y in 621..=626 {
            assert_eq!(geometry.cell(1, y), Some((0, 30)));
            assert_eq!(
                HostPixels { x: 1, y, geometry }.pane_position(full_grid, 1_270, 620),
                None
            );
        }
        assert_eq!(geometry.cell(1_277, 1), None);
        assert_eq!(geometry.cell(1, 627), None);
    }

    #[test]
    fn geometry_rejects_outside_pixels_and_maps_cells() {
        let geometry = HostGeometry::new(80, 24, 800, 480).unwrap();
        assert_eq!(geometry.cell(1, 1), Some((0, 0)));
        assert_eq!(geometry.cell(800, 480), Some((79, 23)));
        assert_eq!(geometry.cell(801, 1), None);
        assert_eq!(geometry.cell(0, 1), None);
    }
}
