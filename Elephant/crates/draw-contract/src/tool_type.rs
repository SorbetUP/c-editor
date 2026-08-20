#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ToolType {
    Selection,
    Lasso,
    Rectangle,
    Diamond,
    Ellipse,
    Arrow,
    Line,
    FreeDraw,
    Text,
    Image,
    Eraser,
    Hand,
    Frame,
    MagicFrame,
    Embeddable,
    Laser,
    Autoshape,
    BucketFill,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolShortcut {
    pub letter_keys: &'static [&'static str],
    pub numeric_key: Option<&'static str>,
    pub shift: bool,
}

impl ToolType {
    pub const ALL: [Self; 18] = [
        Self::Selection,
        Self::Lasso,
        Self::Rectangle,
        Self::Diamond,
        Self::Ellipse,
        Self::Arrow,
        Self::Line,
        Self::FreeDraw,
        Self::Text,
        Self::Image,
        Self::Eraser,
        Self::Hand,
        Self::Frame,
        Self::MagicFrame,
        Self::Embeddable,
        Self::Laser,
        Self::Autoshape,
        Self::BucketFill,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Selection => "selection",
            Self::Lasso => "lasso",
            Self::Rectangle => "rectangle",
            Self::Diamond => "diamond",
            Self::Ellipse => "ellipse",
            Self::Arrow => "arrow",
            Self::Line => "line",
            Self::FreeDraw => "freedraw",
            Self::Text => "text",
            Self::Image => "image",
            Self::Eraser => "eraser",
            Self::Hand => "hand",
            Self::Frame => "frame",
            Self::MagicFrame => "magicframe",
            Self::Embeddable => "embeddable",
            Self::Laser => "laser",
            Self::Autoshape => "autoshape",
            Self::BucketFill => "bucketfill",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tool| tool.id() == id)
    }

    pub const fn shortcut(self) -> ToolShortcut {
        match self {
            Self::Hand => ToolShortcut { letter_keys: &["h"], numeric_key: None, shift: false },
            Self::Selection => ToolShortcut { letter_keys: &["v"], numeric_key: Some("1"), shift: false },
            Self::Lasso => ToolShortcut { letter_keys: &[], numeric_key: None, shift: false },
            Self::Rectangle => ToolShortcut { letter_keys: &["r"], numeric_key: Some("2"), shift: false },
            Self::Diamond => ToolShortcut { letter_keys: &["d"], numeric_key: Some("3"), shift: false },
            Self::Ellipse => ToolShortcut { letter_keys: &["o"], numeric_key: Some("4"), shift: false },
            Self::Arrow => ToolShortcut { letter_keys: &["a"], numeric_key: Some("5"), shift: false },
            Self::Line => ToolShortcut { letter_keys: &["l"], numeric_key: Some("6"), shift: false },
            Self::FreeDraw => ToolShortcut { letter_keys: &["p", "x"], numeric_key: Some("7"), shift: false },
            Self::Text => ToolShortcut { letter_keys: &["t"], numeric_key: Some("8"), shift: false },
            Self::Image => ToolShortcut { letter_keys: &[], numeric_key: Some("9"), shift: false },
            Self::Eraser => ToolShortcut { letter_keys: &["e"], numeric_key: Some("0"), shift: false },
            Self::Frame => ToolShortcut { letter_keys: &["f"], numeric_key: None, shift: false },
            Self::Laser => ToolShortcut { letter_keys: &["k"], numeric_key: None, shift: false },
            Self::Autoshape => ToolShortcut { letter_keys: &["x"], numeric_key: None, shift: true },
            Self::BucketFill => ToolShortcut { letter_keys: &["b"], numeric_key: None, shift: false },
            Self::MagicFrame | Self::Embeddable => ToolShortcut { letter_keys: &[], numeric_key: None, shift: false },
        }
    }

    pub const fn is_transient(self) -> bool {
        matches!(
            self,
            Self::Selection
                | Self::Lasso
                | Self::Hand
                | Self::Eraser
                | Self::Laser
                | Self::Autoshape
                | Self::BucketFill
        )
    }

    pub fn matches_shortcut(self, key: &str, shift: bool) -> bool {
        let shortcut = self.shortcut();
        if shortcut.shift != shift {
            return false;
        }
        let key = key.to_ascii_lowercase();
        shortcut.numeric_key == Some(key.as_str())
            || shortcut
                .letter_keys
                .iter()
                .any(|candidate| *candidate == key)
    }
}
