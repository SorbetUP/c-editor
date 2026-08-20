use crate::DrawingEditor;

impl DrawingEditor {
    pub fn set_selection_locked(&mut self, locked: bool) -> bool {
        let Some(index) = self.selection.index else {
            return false;
        };
        let Some(id) = self
            .scene
            .elements
            .get(index)
            .filter(|element| !element.is_deleted)
            .map(|element| element.id.clone())
        else {
            return false;
        };
        if self.scene.is_element_locked(&id) == locked {
            return false;
        }
        self.history.push(self.scene.clone());
        if !self.scene.set_element_locked(&id, locked) {
            return false;
        }
        if locked {
            self.selection.index = None;
        }
        true
    }

    pub fn unlock_element(&mut self, id: &str) -> bool {
        if !self.scene.is_element_locked(id) {
            return false;
        }
        self.history.push(self.scene.clone());
        if !self.scene.set_element_locked(id, false) {
            return false;
        }
        self.select_by_id(id);
        true
    }

    pub fn resize_selection(&mut self, size: [f32; 2]) -> bool {
        self.mutate_selected(|scene, id| scene.set_element_size(id, size))
    }

    pub fn rotate_selection(&mut self, angle_radians: f32) -> bool {
        self.mutate_selected(|scene, id| scene.rotate_element(id, angle_radians))
    }

    pub fn bring_selection_to_front(&mut self) -> bool {
        self.mutate_selected(|scene, id| scene.bring_element_to_front(id))
    }

    pub fn send_selection_to_back(&mut self) -> bool {
        self.mutate_selected(|scene, id| scene.send_element_to_back(id))
    }

    pub fn bring_selection_forward(&mut self) -> bool {
        self.mutate_selected(|scene, id| scene.bring_element_forward(id))
    }

    pub fn send_selection_backward(&mut self) -> bool {
        self.mutate_selected(|scene, id| scene.send_element_backward(id))
    }

    fn mutate_selected(
        &mut self,
        mutation: impl FnOnce(&mut crate::DrawingScene, &str) -> bool,
    ) -> bool {
        let Some(index) = self.selection.index else {
            return false;
        };
        let Some(id) = self
            .scene
            .elements
            .get(index)
            .filter(|element| !element.is_deleted)
            .map(|element| element.id.clone())
        else {
            return false;
        };
        let before = self.scene.clone();
        if !mutation(&mut self.scene, &id) {
            return false;
        }
        self.history.push(before);
        self.selection.index = self.scene.element_index(&id);
        true
    }
}
