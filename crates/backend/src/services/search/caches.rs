use crate::services::membership::{Albums, MembershipCache, Tags};
use crate::services::search::added::TimelineCache;
use crate::services::search::local_taken::LocalBoundsCache;
use crate::services::search::months::MonthCountCache;

#[derive(Clone, Default)]
pub struct SearchCaches {
    pub(super) timelines: TimelineCache,
    pub(super) months: MonthCountCache,
    pub(super) local_taken: LocalBoundsCache,
    album_membership: MembershipCache<Albums>,
    tag_membership: MembershipCache<Tags>,
}

impl SearchCaches {
    pub fn membership_changed(&self) {
        self.months.membership_changed();
        self.local_taken.invalidate();
        self.names_changed();
    }

    pub fn names_changed(&self) {
        self.album_membership.invalidate();
        self.tag_membership.invalidate();
    }

    pub fn album_membership(&self) -> &MembershipCache<Albums> {
        &self.album_membership
    }

    pub fn tag_membership(&self) -> &MembershipCache<Tags> {
        &self.tag_membership
    }
}
