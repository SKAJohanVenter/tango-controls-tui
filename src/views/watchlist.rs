use crate::views::{Draw, SharedViewState};
use std::convert::{From, Into};
use tui::{backend::Backend, layout::Rect, Frame};

#[derive(Default, Debug)]
pub struct ViewWatchList {
    id: usize,
}

impl ViewWatchList {
    pub fn new(id: usize) -> ViewWatchList {
        ViewWatchList { id }
    }
}

impl Draw for ViewWatchList {
    fn draw_body<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        self.draw_watchlist(f, area);
    }
}

impl From<usize> for ViewWatchList {
    fn from(_item: usize) -> Self {
        ViewWatchList::new(1)
    }
}

impl Into<usize> for ViewWatchList {
    fn into(self) -> usize {
        1
    }
}

#[test]
fn test_watchlist() {
    let id: usize = 1;
    let vwl: ViewWatchList = id.into();
    assert_eq!(vwl.id, 1);

    let id: usize = 5;
    let vwl: ViewWatchList = id.into();
    assert_eq!(vwl.id, 1);

    let id: usize = vwl.into();
    assert_eq!(id, 1);
}
