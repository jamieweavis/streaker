//! Wall-clock scheduling: a tick every minute, stats refreshes with backoff, and the daily streak
//! reminder.

use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDate, TimeDelta, TimeZone, Timelike, Utc};

use crate::contribution::GitHubStats;
use crate::preferences::ReminderTime;

/// Delay after the minute boundary, so a tick never lands just before it due to timer slack.
const TICK_SLACK: Duration = Duration::from_millis(250);

/// How often stats are refreshed while fetches succeed.
const FETCH_INTERVAL: TimeDelta = TimeDelta::minutes(10);
/// The retry delay after a failed fetch, doubling with each further failure up to `MAX_BACKOFF`.
const MIN_BACKOFF: TimeDelta = TimeDelta::minutes(1);
const MAX_BACKOFF: TimeDelta = TimeDelta::hours(1);
/// The longest `Retry-After` honoured, in case GitHub sends something absurd.
const MAX_RETRY_AFTER: TimeDelta = TimeDelta::hours(24);

/// The instant of the next whole wall-clock minute after `now`.
pub fn next_minute<Tz: TimeZone>(now: &DateTime<Tz>) -> Instant {
    let into_minute = Duration::new(u64::from(now.second()), now.nanosecond().min(999_999_999));
    Instant::now() + Duration::from_secs(60).saturating_sub(into_minute) + TICK_SLACK
}

/// Decides when stats are next fetched: every [`FETCH_INTERVAL`], backing off after failures.
/// Uses wall-clock time, so time spent asleep counts towards the wait.
#[derive(Debug, Default)]
pub struct FetchSchedule {
    /// `None` fetches at the next opportunity.
    due: Option<DateTime<Utc>>,
    failures: u32,
}

impl FetchSchedule {
    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        self.due.is_none_or(|due| now >= due)
    }

    /// Fetches at the next opportunity, e.g. after waking from sleep.
    pub fn fetch_now(&mut self) {
        self.due = None;
    }

    pub fn succeeded(&mut self, now: DateTime<Utc>) {
        self.failures = 0;
        self.due = Some(now + FETCH_INTERVAL);
    }

    /// Backs off exponentially, waiting at least as long as GitHub asked to.
    pub fn failed(&mut self, now: DateTime<Utc>, retry_after: Option<Duration>) {
        self.failures = self.failures.saturating_add(1);
        let backoff = (MIN_BACKOFF * 2_i32.pow((self.failures - 1).min(6))).min(MAX_BACKOFF);
        let retry_after = retry_after
            .map_or(TimeDelta::zero(), |delay| TimeDelta::from_std(delay).unwrap_or(MAX_RETRY_AFTER))
            .min(MAX_RETRY_AFTER);
        self.due = Some(now + backoff.max(retry_after));
    }
}

/// Decides when to show the daily reminder, at most once per day.
#[derive(Debug, Default)]
pub struct Reminder {
    last_shown: Option<NaiveDate>,
}

impl Reminder {
    /// Returns the notification to show if the reminder time has passed today, the reminder hasn't
    /// been shown today and the streak is at risk. Checking "at or after" rather than "at" means a
    /// tick that is late or missed (e.g. while asleep) still shows the reminder.
    pub fn check<Tz: TimeZone>(
        &mut self,
        now: &DateTime<Tz>,
        time: ReminderTime,
        stats: Option<&GitHubStats>,
    ) -> Option<(String, String)> {
        if !self.is_pending(now, time) {
            return None;
        }
        // Without stats it can't be decided yet: check again once they arrive.
        let stats = stats?;
        self.last_shown = Some(now.date_naive());

        if !stats.is_streak_at_risk {
            log::info!("Streak not at risk, skipping reminder notification");
            return None;
        }
        Some((
            "🔥 Don't lose your streak!".to_owned(),
            format!("Contribute today to continue your {} day streak on GitHub", stats.previous_streak),
        ))
    }

    /// Whether the reminder time has passed today and the reminder hasn't been decided yet.
    pub fn is_pending<Tz: TimeZone>(&self, now: &DateTime<Tz>, time: ReminderTime) -> bool {
        reached(now, time) && self.last_shown != Some(now.date_naive())
    }
}

/// Whether stats fetched at `fetched_at` are fresh enough to decide today's reminder: fetched since
/// the reminder time, so they include contributions made before it. If the latest fetch failed,
/// stats from earlier today are the best available, so they do.
pub fn can_decide_reminder<Tz: TimeZone>(
    fetched_at: &DateTime<Tz>,
    now: &DateTime<Tz>,
    time: ReminderTime,
    latest_fetch_failed: bool,
) -> bool {
    fetched_at.date_naive() == now.date_naive() && (latest_fetch_failed || reached(fetched_at, time))
}

/// Whether `now` is at or after `time` on its day.
fn reached<Tz: TimeZone>(now: &DateTime<Tz>, time: ReminderTime) -> bool {
    (now.hour(), now.minute()) >= (u32::from(time.hour), u32::from(time.minute))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn at(h: u32, m: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 6, 1, h, m, s).unwrap()
    }

    const EIGHT_PM: ReminderTime = ReminderTime { hour: 20, minute: 0 };

    fn at_risk() -> GitHubStats {
        GitHubStats { is_streak_at_risk: true, previous_streak: 12, ..Default::default() }
    }

    #[test]
    fn next_minute_is_within_a_minute() {
        let before = Instant::now();
        let next = next_minute(&at(10, 0, 59));
        let delay = next - before;
        assert!(delay >= Duration::from_secs(1) && delay <= Duration::from_secs(2), "{delay:?}");
    }

    #[test]
    fn fetches_on_an_interval() {
        let mut schedule = FetchSchedule::default();
        assert!(schedule.is_due(at(10, 0, 0)));

        schedule.succeeded(at(10, 0, 0));
        assert!(!schedule.is_due(at(10, 9, 59)));
        assert!(schedule.is_due(at(10, 10, 0)));

        schedule.fetch_now();
        assert!(schedule.is_due(at(10, 0, 1)));
    }

    #[test]
    fn backs_off_after_failures() {
        let mut schedule = FetchSchedule::default();
        let mut retry_delays = Vec::new();
        for _ in 0..8 {
            schedule.failed(at(10, 0, 0), None);
            retry_delays.push(schedule.due.unwrap() - at(10, 0, 0));
        }
        let minutes: Vec<_> = retry_delays.iter().map(|delay| delay.num_minutes()).collect();
        assert_eq!(minutes, [1, 2, 4, 8, 16, 32, 60, 60]);

        schedule.succeeded(at(10, 0, 0));
        schedule.failed(at(10, 0, 0), None);
        assert_eq!(schedule.due, Some(at(10, 1, 0)));
    }

    #[test]
    fn honours_retry_after() {
        let mut schedule = FetchSchedule::default();
        schedule.failed(at(10, 0, 0), Some(Duration::from_secs(300)));
        assert_eq!(schedule.due, Some(at(10, 5, 0)));

        schedule.failed(at(10, 0, 0), Some(Duration::from_secs(u64::MAX)));
        assert_eq!(schedule.due, Some(at(10, 0, 0) + MAX_RETRY_AFTER));
    }

    #[test]
    fn reminds_once_at_the_reminder_time() {
        let mut reminder = Reminder::default();
        let stats = at_risk();
        assert_eq!(reminder.check(&at(19, 59, 0), EIGHT_PM, Some(&stats)), None);

        let (title, body) = reminder.check(&at(20, 0, 0), EIGHT_PM, Some(&stats)).unwrap();
        assert_eq!(title, "🔥 Don't lose your streak!");
        assert_eq!(body, "Contribute today to continue your 12 day streak on GitHub");

        assert_eq!(reminder.check(&at(20, 0, 30), EIGHT_PM, Some(&stats)), None);
    }

    #[test]
    fn reminds_after_a_missed_reminder_time() {
        let mut reminder = Reminder::default();
        assert!(reminder.check(&at(23, 30, 0), EIGHT_PM, Some(&at_risk())).is_some());
        assert_eq!(reminder.check(&at(23, 31, 0), EIGHT_PM, Some(&at_risk())), None);

        let tomorrow = at(20, 1, 0) + TimeDelta::days(1);
        assert!(reminder.check(&tomorrow, EIGHT_PM, Some(&at_risk())).is_some());
    }

    #[test]
    fn skips_when_not_at_risk() {
        let mut reminder = Reminder::default();
        assert_eq!(reminder.check(&at(20, 0, 0), EIGHT_PM, Some(&GitHubStats::default())), None);
        assert!(!reminder.is_pending(&at(20, 1, 0), EIGHT_PM));
        assert_eq!(reminder.check(&at(20, 1, 0), EIGHT_PM, Some(&at_risk())), None);
    }

    #[test]
    fn waits_for_stats() {
        let mut reminder = Reminder::default();
        assert_eq!(reminder.check(&at(20, 0, 0), EIGHT_PM, None), None);
        assert!(reminder.is_pending(&at(20, 1, 0), EIGHT_PM));
        assert!(reminder.check(&at(20, 1, 0), EIGHT_PM, Some(&at_risk())).is_some());
    }

    #[test]
    fn decides_with_stats_fetched_since_the_reminder_time() {
        let now = at(20, 5, 0);
        assert!(can_decide_reminder(&at(20, 0, 0), &now, EIGHT_PM, false));
        assert!(!can_decide_reminder(&at(19, 59, 0), &now, EIGHT_PM, false));
        // After a failed fetch, earlier stats from today will do, but not yesterday's.
        assert!(can_decide_reminder(&at(9, 0, 0), &now, EIGHT_PM, true));
        assert!(!can_decide_reminder(&(at(21, 0, 0) - TimeDelta::days(1)), &now, EIGHT_PM, true));
    }
}
