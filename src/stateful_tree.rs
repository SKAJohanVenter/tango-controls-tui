use tui_tree_widget::{TreeItem, TreeState};

pub struct StatefulTree<'a> {
    pub state: TreeState<String>,
    pub items: Vec<TreeItem<'a, String>>,
}

impl<'a> StatefulTree<'a> {
    #[allow(dead_code)]
    pub fn new() -> StatefulTree<'a> {
        StatefulTree {
            state: TreeState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<TreeItem<'a, String>>) -> StatefulTree<'a> {
        StatefulTree {
            state: TreeState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        self.state.key_down(&self.items)
    }

    pub fn previous(&mut self) {
        self.state.key_up(&self.items)
    }

    pub fn _close(&mut self) {
        self.state.toggle_selected();
    }

    pub fn left(&mut self) {
        self.state.key_left();
    }

    pub fn right(&mut self) {
        self.state.key_right();
    }

    pub fn _open(&mut self) {
        self.state.open(self.state.selected());
    }
}
