#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingTool {
    Selection,
    Hand,
    Rectangle,
    Diamond,
    Ellipse,
    Arrow,
    Line,
    Freehand,
    Text,
    Image,
    Eraser,
}

impl DrawingTool {
    pub fn id(self) -> &'static str {
        match self {
            Self::Selection => "selection",
            Self::Hand => "hand",
            Self::Rectangle => "rectangle",
            Self::Diamond => "diamond",
            Self::Ellipse => "ellipse",
            Self::Arrow => "arrow",
            Self::Line => "line",
            Self::Freehand => "freedraw",
            Self::Text => "text",
            Self::Image => "image",
            Self::Eraser => "eraser",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "hand" => Self::Hand,
            "rectangle" => Self::Rectangle,
            "diamond" => Self::Diamond,
            "ellipse" => Self::Ellipse,
            "arrow" => Self::Arrow,
            "line" => Self::Line,
            "freedraw" | "freehand" | "pencil" => Self::Freehand,
            "text" => Self::Text,
            "image" => Self::Image,
            "eraser" => Self::Eraser,
            _ => Self::Selection,
        }
    }
}
