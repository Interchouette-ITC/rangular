#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const EMPTY: Self = Self { start: 0, end: 0 };

    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[inline]
pub fn pos(i: usize) -> u32 {
    u32::try_from(i).unwrap_or(u32::MAX)
}

#[must_use]
pub fn line_col(source: &str, offset: u32) -> (u32, u32) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if u32::try_from(i).is_ok_and(|idx| idx >= offset) {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}
