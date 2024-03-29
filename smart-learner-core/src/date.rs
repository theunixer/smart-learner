use chrono::{self, Datelike};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Date {
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

impl PartialEq for Date {
    fn eq(&self, other: &Self) -> bool {
        self.day == other.day && self.month == other.month && self.year == other.year
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.year.partial_cmp(&other.year) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.month.partial_cmp(&other.month) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.day.partial_cmp(&other.day)
    }
}

impl Date {
    pub fn current() -> Self {
        
        let date = chrono::offset::Local::now();

        Date {
            day: date.day() as u8,
            month: date.month() as u8,
            year: date.year() as u16,
        }
    }

    /// Returns difference between 2 dates in days.
    pub fn difference(&self, other: &Self) -> u64 {
        let mut difference: u64 = 0;

        for year in self.year..other.year {
            if is_leap_year(&year) {
                difference += 366;
            } else {
                difference += 365;
            }
        }

        for month in self.month..other.month {
            difference += month_length(&month, &self.year) as u64
        }

        if self.day > other.day {
            difference += (self.day - other.day) as u64;
        } else if other.day > self.day {
            difference += (other.day - self.day) as u64;
        }

        difference
    }
}

const DAYS_IN_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub fn month_length(month: &u8, year: &u16) -> u8 {
    if is_leap_year(year) && *month == 2 {
        29
    } else {
        DAYS_IN_MONTH[*month as usize - 1]
    }
}

pub fn is_leap_year(year: &u16) -> bool {
    ((year % 4 == 0) && (year % 100 != 0)) || (year % 400 == 0)
}
