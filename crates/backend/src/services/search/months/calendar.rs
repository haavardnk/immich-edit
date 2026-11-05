use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc};

use crate::error::AppError;

pub type Span = (DateTime<Utc>, DateTime<Utc>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Month {
    year: i32,
    month: u32,
}

impl Month {
    pub fn of(at: DateTime<Utc>) -> Self {
        Self {
            year: at.year(),
            month: at.month(),
        }
    }

    pub fn year(self) -> i32 {
        self.year
    }

    pub fn label(self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }

    pub fn span(self) -> Result<Span, AppError> {
        Ok((first_instant(self)?, first_instant(self.following())?))
    }

    pub fn through(self, last: Month) -> impl Iterator<Item = Month> {
        std::iter::successors(Some(self), |month| Some(month.following()))
            .take_while(move |month| *month <= last)
    }

    fn following(self) -> Self {
        match self.month {
            12 => Self {
                year: self.year + 1,
                month: 1,
            },
            month => Self {
                year: self.year,
                month: month + 1,
            },
        }
    }
}

pub fn year_span(year: i32) -> Result<Span, AppError> {
    let january = Month { year, month: 1 };
    let next_january = Month {
        year: year + 1,
        month: 1,
    };
    Ok((first_instant(january)?, first_instant(next_january)?))
}

fn first_instant(month: Month) -> Result<DateTime<Utc>, AppError> {
    NaiveDate::from_ymd_opt(month.year, month.month, 1)
        .map(|day| day.and_time(NaiveTime::MIN).and_utc())
        .ok_or(AppError::Internal)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, TimeZone};

    use super::*;

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap()
    }

    #[test]
    fn a_month_start_belongs_to_its_month_and_the_microsecond_before_to_the_previous() {
        let start = at(2024, 4, 1);
        let before = start - TimeDelta::microseconds(1);
        let labels = (Month::of(start).label(), Month::of(before).label());
        if labels != ("2024-04".to_string(), "2024-03".to_string()) {
            panic!("boundary months were {labels:?}");
        }
    }

    #[test]
    fn a_span_runs_from_the_month_start_to_the_next_month_start() {
        let cases = [
            (
                Month::of(at(2023, 12, 31)),
                (at(2023, 12, 1), at(2024, 1, 1)),
            ),
            (Month::of(at(2024, 2, 29)), (at(2024, 2, 1), at(2024, 3, 1))),
        ];
        for (month, expected) in cases {
            let span = month.span().ok();
            if span != Some(expected) {
                panic!("{} spanned {span:?}", month.label());
            }
        }
    }

    #[test]
    fn spans_of_consecutive_months_meet_without_overlap() {
        let months: Vec<Span> = Month::of(at(2023, 11, 5))
            .through(Month::of(at(2024, 2, 5)))
            .filter_map(|month| month.span().ok())
            .collect();
        let touching = months.windows(2).all(|pair| pair[0].1 == pair[1].0);
        if months.len() != 4 || !touching {
            panic!("consecutive spans were {months:?}");
        }
    }

    #[test]
    fn through_walks_oldest_first_across_years_and_stops_at_the_last_month() {
        let labels: Vec<String> = Month::of(at(2023, 11, 5))
            .through(Month::of(at(2024, 2, 5)))
            .map(Month::label)
            .collect();
        if labels != ["2023-11", "2023-12", "2024-01", "2024-02"] {
            panic!("walked {labels:?}");
        }
        let reversed = Month::of(at(2024, 2, 5)).through(Month::of(at(2023, 11, 5)));
        if reversed.count() != 0 {
            panic!("a reversed walk yielded months");
        }
    }

    #[test]
    fn a_year_span_covers_january_to_the_next_january() {
        let span = year_span(2020).ok();
        if span != Some((at(2020, 1, 1), at(2021, 1, 1))) {
            panic!("year spanned {span:?}");
        }
    }

    #[test]
    fn labels_are_zero_padded() {
        let label = Month::of(at(987, 3, 1)).label();
        if label != "0987-03" {
            panic!("label was {label}");
        }
    }
}
