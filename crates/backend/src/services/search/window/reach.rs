use crate::error::AppError;
use crate::services::search::query::Direction;

pub const MAX_RADIUS: u32 = 250;
const MAX_SIDE: u32 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reach {
    pub before: u32,
    pub after: u32,
}

impl Reach {
    pub fn parse(
        radius: Option<u32>,
        before: Option<u32>,
        after: Option<u32>,
    ) -> Result<Self, AppError> {
        if radius.is_some_and(|radius| !(1..=MAX_RADIUS).contains(&radius)) {
            return Err(AppError::BadRequest(format!(
                "radius must be 1 to {MAX_RADIUS}"
            )));
        }
        if [before, after]
            .into_iter()
            .flatten()
            .any(|side| side > MAX_SIDE)
        {
            return Err(AppError::BadRequest(format!(
                "before and after must be 0 to {MAX_SIDE}"
            )));
        }
        if radius.is_none() && before.is_none() && after.is_none() {
            return Err(AppError::BadRequest(
                "a window needs radius, before or after".into(),
            ));
        }
        Ok(Self {
            before: before.or(radius).unwrap_or(0),
            after: after.or(radius).unwrap_or(0),
        })
    }

    pub fn greater_and_lesser(self, direction: Direction) -> (usize, usize) {
        match direction {
            Direction::Desc => (self.before as usize, self.after as usize),
            Direction::Asc => (self.after as usize, self.before as usize),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reach(before: u32, after: u32) -> Reach {
        Reach { before, after }
    }

    #[test]
    fn radius_fills_the_sides_that_are_missing() {
        let cases = [
            ((Some(5), None, None), reach(5, 5)),
            ((Some(5), Some(2), None), reach(2, 5)),
            ((Some(5), None, Some(0)), reach(5, 0)),
            ((Some(5), Some(500), Some(1)), reach(500, 1)),
            ((None, Some(3), None), reach(3, 0)),
            ((None, None, Some(7)), reach(0, 7)),
            ((None, Some(0), Some(0)), reach(0, 0)),
        ];
        for ((radius, before, after), expected) in cases {
            let parsed = Reach::parse(radius, before, after).ok();
            if parsed != Some(expected) {
                panic!("{radius:?} {before:?} {after:?} parsed as {parsed:?}");
            }
        }
    }

    #[test]
    fn out_of_range_or_empty_reaches_are_rejected() {
        let cases = [
            (None, None, None),
            (Some(0), None, None),
            (Some(251), Some(1), Some(1)),
            (None, Some(501), None),
            (Some(10), None, Some(501)),
        ];
        for (radius, before, after) in cases {
            if !matches!(
                Reach::parse(radius, before, after),
                Err(AppError::BadRequest(_))
            ) {
                panic!("{radius:?} {before:?} {after:?} should be a bad request");
            }
        }
    }

    #[test]
    fn descending_puts_greater_keys_before_the_anchor() {
        let split = reach(2, 9);
        if split.greater_and_lesser(Direction::Desc) != (2, 9) {
            panic!(
                "descending split was {:?}",
                split.greater_and_lesser(Direction::Desc)
            );
        }
        if split.greater_and_lesser(Direction::Asc) != (9, 2) {
            panic!(
                "ascending split was {:?}",
                split.greater_and_lesser(Direction::Asc)
            );
        }
    }
}
