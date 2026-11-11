//! Streak and contribution stats scraped from a user's GitHub contribution graph.
//!
//! The GitHub API doesn't expose contribution streaks, so this parses the graph rendered at
//! `https://github.com/users/<username>/contributions`.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Context as _, bail};
use chrono::{NaiveDate, Utc};
use scraper::{Html, Selector};

/// Stats for the days on the graph, up to and including today. GitHub assigns contributions to
/// days in UTC, so "today" is the current UTC date rather than the local one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitHubStats {
    /// Longest run of consecutive days with contributions.
    pub best_streak: u32,
    /// Run of consecutive days with contributions ending today.
    pub current_streak: u32,
    /// Length of the most recent streak that has been broken (or is at risk of breaking).
    pub previous_streak: u32,
    /// No contributions today but some yesterday: the streak is lost without one today.
    pub is_streak_at_risk: bool,
    pub most_contributions: u32,
    pub todays_contributions: u32,
    pub total_contributions: u32,
}

/// 1-39 ASCII alphanumerics or single hyphens, not starting or ending with a hyphen.
pub fn is_valid_username(username: &str) -> bool {
    (1..=39).contains(&username.len())
        && username.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        && !username.starts_with('-')
        && !username.ends_with('-')
        && !username.contains("--")
}

/// The HTTP client for requests to GitHub, shared so connections are reused between requests.
pub fn http_agent() -> &'static ureq::Agent {
    static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .user_agent(concat!("streaker/", env!("CARGO_PKG_VERSION")))
            .build()
            .into()
    });
    &AGENT
}

/// Fetches a user's contribution graph and calculates their stats. Blocks until done.
pub fn fetch_github_stats(username: &str) -> anyhow::Result<GitHubStats> {
    let html = fetch_graph(username)?;
    let days = parse_daily_contributions(&html)?;
    Ok(calculate_stats(&daily_counts_until(&days, Utc::now().date_naive())))
}

/// Fetches the HTML of a user's contribution graph. Blocks until done.
fn fetch_graph(username: &str) -> anyhow::Result<String> {
    if !is_valid_username(username) {
        bail!("\"{username}\" is not a valid GitHub username");
    }

    let response = http_agent()
        .get(format!("https://github.com/users/{username}/contributions"))
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .context("failed to fetch contribution graph")?;
    match response.status().as_u16() {
        404 => bail!("GitHub user \"{username}\" was not found"),
        429 => return Err(RateLimited { retry_after: retry_after(&response) }.into()),
        status if !response.status().is_success() => bail!("GitHub returned HTTP {status}"),
        _ => {}
    }
    Ok(response.into_body().read_to_string()?)
}

/// GitHub refused a request because too many have been made.
#[derive(Debug)]
pub struct RateLimited {
    /// How long GitHub asked to wait before retrying.
    pub retry_after: Option<Duration>,
}

impl fmt::Display for RateLimited {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("rate limited by GitHub")
    }
}

impl std::error::Error for RateLimited {}

/// The `Retry-After` header, in its delay-seconds form (GitHub doesn't send HTTP dates).
fn retry_after<B>(response: &ureq::http::Response<B>) -> Option<Duration> {
    let value = response.headers().get(ureq::http::header::RETRY_AFTER)?;
    value.to_str().ok()?.trim().parse().ok().map(Duration::from_secs)
}

/// A failed stats fetch: the message to show, and how long GitHub asked to wait, if it did.
#[derive(Debug)]
pub struct FetchError {
    pub message: String,
    pub retry_after: Option<Duration>,
}

impl From<anyhow::Error> for FetchError {
    fn from(e: anyhow::Error) -> Self {
        let retry_after = e.downcast_ref::<RateLimited>().and_then(|r| r.retry_after);
        Self { message: format!("{e:#}"), retry_after }
    }
}

/// A graph cell, e.g. `<td ... data-date="2025-10-26" id="contribution-day-component-0-2" ...>`.
static DAY_CELL: LazyLock<Selector> = LazyLock::new(|| Selector::parse("td[data-date]").unwrap());
/// A cell's tooltip, e.g. `<tool-tip ... for="contribution-day-component-0-2" ...>10 contributions on ...`.
static TOOLTIP: LazyLock<Selector> = LazyLock::new(|| Selector::parse("tool-tip[for]").unwrap());

/// The count from a tooltip's text, e.g. 1234 from "1,234 contributions on January 3rd.", or
/// `None` if it isn't a contribution count.
fn tooltip_count(text: &str) -> Option<u32> {
    let mut words = text.split_whitespace();
    let count = words.next()?;
    if !words.next()?.starts_with("contribution") {
        return None;
    }
    if count == "No" { Some(0) } else { count.replace(',', "").parse().ok() }
}

/// Returns the contribution count for each day on the graph.
///
/// Each cell carries its date, while the count lives in a tooltip linked to the cell by id.
fn parse_daily_contributions(html: &str) -> anyhow::Result<BTreeMap<NaiveDate, u32>> {
    let document = Html::parse_document(html);
    let counts: HashMap<&str, u32> = document
        .select(&TOOLTIP)
        .filter_map(|tooltip| {
            let count = tooltip_count(&tooltip.text().collect::<String>())?;
            Some((tooltip.attr("for")?, count))
        })
        .collect();

    let mut days = BTreeMap::new();
    for cell in document.select(&DAY_CELL) {
        let date = cell.attr("data-date").unwrap_or_default();
        let id = cell.attr("id").with_context(|| format!("no id for day cell {date}"))?;
        let count = counts.get(id).with_context(|| format!("no tooltip for {date}"))?;
        let date =
            NaiveDate::parse_from_str(date, "%Y-%m-%d").with_context(|| format!("invalid date {date}"))?;
        days.insert(date, *count);
    }

    if days.is_empty() {
        bail!("no contribution days found on the graph");
    }
    Ok(days)
}

/// Returns daily contribution counts from the first day on the graph to `today`, oldest first.
///
/// The graph may not have caught up with `today` yet (e.g. a cached page just after midnight
/// UTC), so days missing from it count as having no contributions. Days after `today` are dropped.
fn daily_counts_until(days: &BTreeMap<NaiveDate, u32>, today: NaiveDate) -> Vec<u32> {
    let Some(&first) = days.keys().next() else {
        return Vec::new();
    };
    first
        .iter_days()
        .take_while(|day| *day <= today)
        .map(|day| days.get(&day).copied().unwrap_or(0))
        .collect()
}

/// Calculates stats from daily contribution counts, oldest first.
fn calculate_stats(daily_contributions: &[u32]) -> GitHubStats {
    let mut stats = GitHubStats::default();
    let mut yesterdays_streak = 0;

    for &contributions in daily_contributions {
        stats.total_contributions += contributions;
        stats.todays_contributions = contributions;
        stats.most_contributions = stats.most_contributions.max(contributions);

        stats.current_streak = if contributions > 0 { stats.current_streak + 1 } else { 0 };
        stats.best_streak = stats.best_streak.max(stats.current_streak);

        stats.is_streak_at_risk = stats.current_streak == 0 && yesterdays_streak > 0;
        if stats.is_streak_at_risk {
            stats.previous_streak = yesterdays_streak;
        }
        yesterdays_streak = stats.current_streak;
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_usernames() {
        for valid in ["jamieweavis", "a-b-c", "A1"] {
            assert!(is_valid_username(valid), "{valid}");
        }
        for invalid in ["", "-abc", "abc-", "a--b", "a/b", "a b", &"a".repeat(40)] {
            assert!(!is_valid_username(invalid), "{invalid}");
        }
    }

    #[test]
    fn parses_cells_and_tooltips_by_id() {
        let html = r#"
            <table><tr>
                <td data-ix="0" data-date="2025-01-02" id="day-b" data-level="1"></td>
                <td data-ix="1" data-date="2025-01-01" id="day-a" data-level="0"></td>
                <td data-ix="2" id="day-c" data-level="4" data-date="2025-01-03"></td>
            </tr></table>
            <tool-tip for="day-a" class="sr-only">No contributions on January 1st.</tool-tip>
            <tool-tip id="t" for="day-c">1,234 contributions on January 3rd.</tool-tip>
            <tool-tip for="day-b">1 contribution on January 2nd.</tool-tip>
            <tool-tip for="legend">Less</tool-tip>
        "#;
        let days = parse_daily_contributions(html).unwrap();
        assert_eq!(days.keys().next(), Some(&NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()));
        assert_eq!(days.into_values().collect::<Vec<_>>(), [0, 1, 1234]);
    }

    #[test]
    fn matches_attributes_by_exact_name() {
        let html = r#"
            <table><tr><td data-id="wrong" data-date="2025-01-01" id="day-a"></td></tr></table>
            <tool-tip data-for="day-a" for="day-a">2 contributions on January 1st.</tool-tip>
            <tool-tip data-for="day-a" for="other">5 contributions on January 1st.</tool-tip>
        "#;
        assert_eq!(parse_daily_contributions(html).unwrap().into_values().collect::<Vec<_>>(), [2]);
    }

    #[test]
    fn parses_tooltip_counts() {
        assert_eq!(tooltip_count("  No contributions on January 1st."), Some(0));
        assert_eq!(tooltip_count("1 contribution on January 2nd."), Some(1));
        assert_eq!(tooltip_count("1,234 contributions on January 3rd."), Some(1234));
        assert_eq!(tooltip_count("Less"), None);
        assert_eq!(tooltip_count("3 reviews"), None);
    }

    #[test]
    fn rejects_malformed_graphs() {
        let missing_tooltip = r#"<table><tr><td data-date="2025-01-01" id="day-a"></td></tr></table>"#;
        assert!(parse_daily_contributions(missing_tooltip).is_err());
        let missing_id = r#"<table><tr><td data-date="2025-01-01"></td></tr></table>"#;
        assert!(parse_daily_contributions(missing_id).is_err());
        assert!(parse_daily_contributions("<html></html>").is_err());
    }

    #[test]
    fn counts_days_until_today() {
        let date = |day| NaiveDate::from_ymd_opt(2025, 1, day).unwrap();
        let days = BTreeMap::from([(date(1), 1), (date(2), 2), (date(4), 4), (date(5), 5)]);
        // A gap is filled, and days after today are dropped.
        assert_eq!(daily_counts_until(&days, date(4)), [1, 2, 0, 4]);
        // The graph hasn't caught up with today yet.
        assert_eq!(daily_counts_until(&days, date(7)), [1, 2, 0, 4, 5, 0, 0]);
        assert!(daily_counts_until(&BTreeMap::new(), date(1)).is_empty());
    }

    #[test]
    fn counts_contributions() {
        let stats = calculate_stats(&[3, 0, 7, 2]);
        assert_eq!(
            (stats.total_contributions, stats.most_contributions, stats.todays_contributions),
            (12, 7, 2)
        );
    }

    #[test]
    fn current_best_and_previous_streaks() {
        let stats = calculate_stats(&[1, 1, 1, 0, 1, 1]);
        assert_eq!((stats.current_streak, stats.best_streak, stats.previous_streak), (2, 3, 3));
        assert!(!stats.is_streak_at_risk);

        let stats = calculate_stats(&[1, 1, 1, 1, 0, 1, 0, 1]);
        assert_eq!((stats.current_streak, stats.best_streak, stats.previous_streak), (1, 4, 1));
    }

    #[test]
    fn streak_at_risk_only_on_first_empty_day() {
        let stats = calculate_stats(&[0, 1, 1, 1, 1, 0]);
        assert_eq!((stats.current_streak, stats.previous_streak), (0, 4));
        assert!(stats.is_streak_at_risk);

        assert!(!calculate_stats(&[1, 1, 0, 0]).is_streak_at_risk);
    }

    /// A real graph captured from github.com. Expected values come from the original JavaScript
    /// `contribution` package run on the same page.
    #[test]
    fn matches_javascript_implementation_on_real_graph() {
        let days = parse_daily_contributions(include_str!("fixtures/contributions.html")).unwrap();
        assert_eq!(days.len(), 365);
        let today = *days.keys().next_back().unwrap();
        assert_eq!(
            calculate_stats(&daily_counts_until(&days, today)),
            GitHubStats {
                best_streak: 35,
                current_streak: 1,
                previous_streak: 14,
                is_streak_at_risk: false,
                most_contributions: 52,
                todays_contributions: 8,
                total_contributions: 2134,
            }
        );
    }

    /// The same graph fetched the next UTC day, before GitHub's page has a cell for it: the
    /// streak is at risk rather than continuing.
    #[test]
    fn real_graph_behind_today_puts_streak_at_risk() {
        let days = parse_daily_contributions(include_str!("fixtures/contributions.html")).unwrap();
        let tomorrow = days.keys().next_back().unwrap().succ_opt().unwrap();
        let stats = calculate_stats(&daily_counts_until(&days, tomorrow));
        assert_eq!((stats.current_streak, stats.todays_contributions, stats.previous_streak), (0, 0, 1));
        assert!(stats.is_streak_at_risk);
    }

    #[test]
    fn fetch_error_keeps_retry_after() {
        let error =
            FetchError::from(anyhow::Error::new(RateLimited { retry_after: Some(Duration::from_secs(90)) }));
        assert_eq!(error.retry_after, Some(Duration::from_secs(90)));
        assert_eq!(error.message, "rate limited by GitHub");
        assert_eq!(FetchError::from(anyhow::anyhow!("offline")).retry_after, None);
    }

    #[test]
    fn rejects_invalid_username_without_requesting() {
        assert!(fetch_github_stats("../evil").is_err());
    }

    /// Fetches a live graph from github.com to catch GitHub changing its markup. Ignored by
    /// default as it needs the network; run daily in CI by `.github/workflows/graph.yml`.
    #[test]
    #[ignore = "requests github.com"]
    fn parses_live_graph() {
        let days = parse_daily_contributions(&fetch_graph("jamieweavis").unwrap()).unwrap();
        // A year of days, plus up to a week to fill the first column.
        assert!((365..=372).contains(&days.len()), "{} days", days.len());
        let today = Utc::now().date_naive();
        let latest = *days.keys().next_back().unwrap();
        assert!((today - latest).num_days().abs() <= 1, "latest day {latest}, today {today}");
        assert!(days.values().any(|&count| count > 0), "no contributions found");
    }
}
