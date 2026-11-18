use crate::services::search::added::TimelineCache;
use crate::services::search::months::MonthCountCache;

#[derive(Clone, Default)]
pub struct SearchCaches {
    pub(super) timelines: TimelineCache,
    pub(super) months: MonthCountCache,
}

impl SearchCaches {
    pub fn membership_changed(&self) {
        self.months.membership_changed();
    }
}
