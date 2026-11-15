use crate::model::StreamRow;
use crate::slint_ui::state::SlintApp;

pub fn open(a: &mut SlintApp, _row: &StreamRow) {
    a.toast("Series view: step 6");
}

pub fn open_details(a: &mut SlintApp, _row: &StreamRow) {
    a.toast("Details: step 6");
}

pub fn close(_a: &mut SlintApp) {}
