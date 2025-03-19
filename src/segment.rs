use crate::{
    config::Style,
    print::{Grapheme, UnicodeWidthGraphemes},
};
use nu_ansi_term::{AnsiString, Style as AnsiStyle};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug)]
pub struct SeparatorSegment {
    /// The segment's style. If None, will inherit the style of the module containing it.
    style: Option<Style>,

    /// The string value of the current segment.
    value: String,

    /// The separator direction
    left: bool,
}

impl SeparatorSegment {
    // Returns the AnsiString of the segment value
    pub fn ansi_string(&self, prev: Option<&AnsiStyle>) -> AnsiString {
        match self.style {
            Some(style) => style.to_ansi_style(prev).paint(&self.value),
            None => AnsiString::from(&self.value),
        }
    }

    pub fn is_left(&self) -> bool {
        self.left
    }

    pub fn derive_style(&self, prev: Option<AnsiStyle>, next: Option<AnsiStyle>) -> AnsiStyle {
        let mut resulting_style = AnsiStyle::default();
        match (prev, next) {
            (Some(prev), Some(next)) => {
                resulting_style.foreground = next.background;
                resulting_style.background = prev.background;
                resulting_style
            }
            (Some(prev), None) => {
                resulting_style.foreground = prev.background;
                resulting_style
            }
            (None, Some(next)) => {
                resulting_style.foreground = next.background;
                resulting_style
            }
            (None, None) => AnsiStyle::default(),
        }
    }

    pub fn set_style(&mut self, style: Option<AnsiStyle>) {
        self.style = style.map(|s| s.into());
    }
}
/// Type that holds text with an associated style
#[derive(Clone, Debug)]
pub struct TextSegment {
    /// The segment's style. If None, will inherit the style of the module containing it.
    style: Option<Style>,

    /// The string value of the current segment.
    value: String,
}

impl TextSegment {
    // Returns the AnsiString of the segment value
    fn ansi_string(&self, prev: Option<&AnsiStyle>) -> AnsiString {
        match self.style {
            Some(style) => style.to_ansi_style(prev).paint(&self.value),
            None => AnsiString::from(&self.value),
        }
    }
}

/// Type that holds fill text with an associated style
#[derive(Clone, Debug)]
pub struct FillSegment {
    /// The segment's style. If None, will inherit the style of the module containing it.
    style: Option<Style>,

    /// The string value of the current segment.
    value: String,
}

impl FillSegment {
    // Returns the AnsiString of the segment value, not including its prefix and suffix
    pub fn ansi_string(&self, width: Option<usize>, prev: Option<&AnsiStyle>) -> AnsiString {
        let s = match width {
            Some(w) => self
                .value
                .graphemes(true)
                .cycle()
                .scan(0usize, |len, g| {
                    *len += Grapheme(g).width();
                    if *len <= w { Some(g) } else { None }
                })
                .collect::<String>(),
            None => String::from(&self.value),
        };
        match self.style {
            Some(style) => style.to_ansi_style(prev).paint(s),
            None => AnsiString::from(s),
        }
    }
}

#[cfg(test)]
mod fill_seg_tests {
    use super::FillSegment;
    use nu_ansi_term::Color;

    #[test]
    fn ansi_string_width() {
        let width: usize = 10;
        let style = Color::Blue.bold();

        let inputs = vec![
            (".", ".........."),
            (".:", ".:.:.:.:.:"),
            ("-:-", "-:--:--:--"),
            ("🟦", "🟦🟦🟦🟦🟦"),
            ("🟢🔵🟡", "🟢🔵🟡🟢🔵"),
        ];

        for (text, expected) in &inputs {
            let f = FillSegment {
                value: String::from(*text),
                style: Some(style.into()),
            };
            let actual = f.ansi_string(Some(width), None);
            assert_eq!(style.paint(*expected), actual);
        }
    }
}

/// A segment is a styled text chunk ready for printing.
#[derive(Clone, Debug)]
pub enum Segment {
    Text(TextSegment),
    Fill(FillSegment),
    Separator(SeparatorSegment),
    LineTerm,
}

impl Segment {
    /// Creates new segments from a text with a style; breaking out `LineTerminators`.
    pub fn from_text<T>(style: Option<Style>, value: T) -> Vec<Self>
    where
        T: Into<String>,
    {
        let mut segs: Vec<Self> = Vec::new();
        value.into().split(LINE_TERMINATOR).for_each(|s| {
            if !segs.is_empty() {
                segs.push(Self::LineTerm)
            }
            segs.push(Self::Text(TextSegment {
                value: String::from(s),
                style,
            }))
        });
        segs
    }

    /// Creates a new fill segment
    pub fn fill<T>(style: Option<Style>, value: T) -> Self
    where
        T: Into<String>,
    {
        Self::Fill(FillSegment {
            style,
            value: value.into(),
        })
    }

    /// Creates a new separator segment
    pub fn separator<T>(style: Option<Style>, value: T) -> Self
    where
        T: Into<String>,
    {
        let s = value.into();
        let left = s.ends_with("l");
        let symbol = s.trim_end_matches(['l', 'r']);
        Self::Separator(SeparatorSegment {
            style,
            value: String::from(symbol),
            left,
        })
    }

    pub fn style(&self) -> Option<AnsiStyle> {
        match self {
            Self::Fill(fs) => fs.style.map(|cs| cs.to_ansi_style(None)),
            Self::Text(ts) => ts.style.map(|cs| cs.to_ansi_style(None)),
            Self::Separator(ss) => ss.style.map(|cs| cs.to_ansi_style(None)),
            Self::LineTerm => None,
        }
    }

    pub fn set_style_if_empty(&mut self, style: Option<Style>) {
        match self {
            Self::Fill(fs) => {
                if fs.style.is_none() {
                    fs.style = style
                }
            }
            Self::Text(ts) => {
                if ts.style.is_none() {
                    ts.style = style
                }
            }
            Self::Separator(ss) => {
                if ss.style.is_none() {
                    ss.style = style
                }
            }
            Self::LineTerm => {}
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Fill(fs) => &fs.value,
            Self::Text(ts) => &ts.value,
            Self::Separator(ss) => &ss.value,
            Self::LineTerm => LINE_TERMINATOR_STRING,
        }
    }

    // Returns the AnsiString of the segment value, not including its prefix and suffix
    pub fn ansi_string(&self, prev: Option<&AnsiStyle>) -> AnsiString {
        match self {
            Self::Fill(fs) => fs.ansi_string(None, prev),
            Self::Text(ts) => ts.ansi_string(prev),
            Self::Separator(ss) => ss.ansi_string(prev),
            Self::LineTerm => AnsiString::from(LINE_TERMINATOR_STRING),
        }
    }

    pub fn width_graphemes(&self) -> usize {
        match self {
            Self::Fill(fs) => fs.value.width_graphemes(),
            Self::Text(ts) => ts.value.width_graphemes(),
            Self::Separator(ss) => ss.value.width_graphemes(),
            Self::LineTerm => 0,
        }
    }

    pub fn is_linebreak(&self) -> bool {
        matches!(self, Self::LineTerm)
    }
}

const LINE_TERMINATOR: char = '\n';
const LINE_TERMINATOR_STRING: &str = "\n";
