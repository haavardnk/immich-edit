use crate::services::search::added::TimelineCache;
use crate::services::search::local_taken::LocalBoundsCache;
use crate::services::search::months::MonthCountCache;

#[derive(Clone, Default)]
pub struct SearchCaches {
    pub(super) timelines: TimelineCache,
    pub(super) months: MonthCountCache,
    pub(super) local_taken: LocalBoundsCache,
}

impl SearchCaches {
    pub fn membership_changed(&self) {
        self.months.membership_changed();
        self.local_taken.membership_changed();
    }
}
