use chrono::{DateTime, Days, Duration, LocalResult, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;

const LOCAL_DATETIME_FORMATS: [&str; 8] = [
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%d %H:%M",
    "%Y/%m/%d %H:%M:%S",
    "%Y/%m/%d %H:%M",
    "%Y-%m-%d %I:%M %p",
    "%Y/%m/%d %I:%M %p",
    "%Y-%m-%dT%H:%M:%S",
    "%Y-%m-%dT%H:%M",
];
const LOCAL_TIME_FORMATS: [&str; 4] = ["%H:%M", "%H:%M:%S", "%I:%M %p", "%I:%M%p"];
const HUMAN_DISPLAY_FORMAT: &str = "%Y-%m-%d %I:%M %p %Z";

pub(super) fn normalize_scheduled_time_input(raw: &str, time_zone: Tz) -> Result<String, String> {
    let input = raw.trim();
    if input.is_empty() {
        return Err("scheduled time cannot be empty".to_string());
    }

    if let Ok(parsed) = DateTime::parse_from_rfc3339(input) {
        return Ok(parsed.with_timezone(&Utc).to_rfc3339());
    }

    let now_utc = Utc::now();
    let now_local = now_utc.with_timezone(&time_zone);

    if let Some(relative_dt) = parse_relative_time(input, now_local) {
        return Ok(relative_dt.with_timezone(&Utc).to_rfc3339());
    }

    if let Some(local_dt) = parse_local_datetime(input, time_zone) {
        return Ok(local_dt.with_timezone(&Utc).to_rfc3339());
    }

    if let Some(local_dt) = parse_local_time_only(input, time_zone, now_local) {
        return Ok(local_dt.with_timezone(&Utc).to_rfc3339());
    }

    Err(format!(
        "unsupported scheduled time '{input}'; use local time like '2026-02-25 10:09 PM', '2026-02-25 22:09', '22:09', or 'in 30 minutes'"
    ))
}

pub(super) fn render_scheduled_time_local(raw: &str, time_zone: Tz) -> String {
    if raw.eq_ignore_ascii_case("unscheduled") {
        return "Unscheduled".to_string();
    }

    if let Ok(parsed) = DateTime::parse_from_rfc3339(raw) {
        return parsed
            .with_timezone(&time_zone)
            .format(HUMAN_DISPLAY_FORMAT)
            .to_string();
    }

    let normalized = normalize_scheduled_time_input(raw, time_zone)
        .ok()
        .and_then(|value| DateTime::parse_from_rfc3339(&value).ok());
    match normalized {
        Some(parsed) => parsed
            .with_timezone(&time_zone)
            .format(HUMAN_DISPLAY_FORMAT)
            .to_string(),
        None => raw.to_string(),
    }
}

fn parse_relative_time(input: &str, now_local: DateTime<Tz>) -> Option<DateTime<Tz>> {
    let lowered = input.trim().to_ascii_lowercase();
    let payload = lowered.strip_prefix("in ")?;
    let mut parts = payload.split_whitespace();
    let amount = parts.next()?.parse::<i64>().ok()?;
    let unit = parts.next()?;
    if parts.next().is_some() || amount <= 0 {
        return None;
    }

    match unit {
        "m" | "min" | "mins" | "minute" | "minutes" => Some(now_local + Duration::minutes(amount)),
        "h" | "hr" | "hrs" | "hour" | "hours" => Some(now_local + Duration::hours(amount)),
        "d" | "day" | "days" => Some(now_local + Duration::days(amount)),
        _ => None,
    }
}

fn parse_local_datetime(input: &str, time_zone: Tz) -> Option<DateTime<Tz>> {
    LOCAL_DATETIME_FORMATS
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(input, format).ok())
        .and_then(|naive| resolve_local_datetime(time_zone, naive))
}

fn parse_local_time_only(
    input: &str,
    time_zone: Tz,
    now_local: DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    let parsed_time = LOCAL_TIME_FORMATS
        .iter()
        .find_map(|format| NaiveTime::parse_from_str(input, format).ok())?;
    let today_naive = now_local.date_naive().and_time(parsed_time);
    let mut candidate = resolve_local_datetime(time_zone, today_naive)?;
    if candidate <= now_local {
        let next_day = now_local.date_naive().checked_add_days(Days::new(1))?;
        let next_naive = next_day.and_time(parsed_time);
        candidate = resolve_local_datetime(time_zone, next_naive)?;
    }
    Some(candidate)
}

fn resolve_local_datetime(time_zone: Tz, naive: NaiveDateTime) -> Option<DateTime<Tz>> {
    match time_zone.from_local_datetime(&naive) {
        LocalResult::Single(dt) => Some(dt),
        LocalResult::Ambiguous(first, _) => Some(first),
        LocalResult::None => None,
    }
}
