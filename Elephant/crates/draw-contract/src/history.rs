use crate::DrawingScene;

const DEFAULT_HISTORY_LIMIT: usize = 100;

#[derive(Clone, Debug, PartialEq)]
pub struct HistoryState {
    undo: Vec<DrawingScene>,
    redo: Vec<DrawingScene>,
    limit: usize,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: DEFAULT_HISTORY_LIMIT,
        }
    }
}

impl HistoryState {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            ..Self::default()
        }
    }

    pub fn push(&mut self, scene: DrawingScene) {
        if self.undo.last() == Some(&scene) {
            return;
        }
        self.undo.push(scene);
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn undo(&mut self, current: DrawingScene) -> Option<DrawingScene> {
        let previous = self.undo.pop()?;
        self.redo.push(current);
        Some(previous)
    }

    pub fn redo(&mut self, current: DrawingScene) -> Option<DrawingScene> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        Some(next)
    }

    pub fn rollback(&mut self) -> Option<DrawingScene> {
        let previous = self.undo.pop()?;
        self.redo.clear();
        Some(previous)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
