use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled},
    widgets::Widget,
};

const BAR_PROGESS_CHAR: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

fn cal_chars(max_height: usize, ratio: f64) -> Vec<&'static str> {
    let height = max_height as f64 * ratio;
    let full_count = height.trunc() as usize;

    let mut chars = vec![BAR_PROGESS_CHAR[0]; max_height];

    for i in chars.iter_mut().take(full_count) {
        *i = BAR_PROGESS_CHAR[8];
    }

    let fract = height.fract();
    if fract > 0.0 {
        let char_index = (fract * 8.0).round() as usize;
        chars[full_count] = BAR_PROGESS_CHAR[char_index];
    }

    chars.reverse();
    chars
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct VerticalGauge {
    ratio: f64,
    style: Style,
}

impl VerticalGauge {
    /// Sets the bar progression from a ratio (float).
    ///
    /// `ratio` is the ratio between filled bar over empty bar (i.e. `3/4` completion is `0.75`).
    /// This is more easily seen as a floating point percentage (e.g. 42% = `0.42`).
    ///
    /// # Panics
    ///
    /// This method panics if `ratio` is **not** between 0 and 1 inclusively.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn ratio(mut self, ratio: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&ratio),
            "Ratio should be between 0 and 1 inclusively."
        );
        self.ratio = ratio;
        self
    }

    /// Sets the widget style.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// This will style everything except the bar itself, so basically the block (if any) and
    /// background.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
}

impl Widget for VerticalGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        if area.is_empty() {
            return;
        }

        let start = area.top();
        if start >= area.bottom() {
            return;
        }
        let col = area.left();

        let chars = cal_chars(area.height as usize, self.ratio);

        for (row, char) in chars.into_iter().enumerate() {
            buf[(col, row as u16)].set_symbol(char);
        }
    }
}

impl Styled for VerticalGauge {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

mod tests {
    use std::fmt;

    use super::*;

    #[test]
    fn test_cal_chars() {
        assert_vec_eq(cal_chars(4, 1.0), ["█", "█", "█", "█"].to_vec());
        assert_vec_eq(cal_chars(4, 0.75), [" ", "█", "█", "█"].to_vec());
        assert_vec_eq(cal_chars(4, 0.5), [" ", " ", "█", "█"].to_vec());
        assert_vec_eq(cal_chars(4, 0.25), [" ", " ", " ", "█"].to_vec());

        assert_vec_eq(cal_chars(4, 0.125), [" ", " ", " ", "▄"].to_vec());
    }

    fn assert_vec_eq<T: PartialEq + fmt::Debug>(vec1: Vec<T>, vec2: Vec<T>) {
        assert_eq!(vec1.len(), vec2.len());
        for i in 0..vec1.len() {
            assert_eq!(vec1[i], vec2[i]);
        }
    }
}
