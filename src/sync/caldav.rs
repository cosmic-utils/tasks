use base64::Engine as _;
use chrono::{DateTime, NaiveDate, Utc};
use icalendar::{Calendar as ICalendar, CalendarDateTime, Component, DatePerhapsTime, Todo};
use quick_xml::Reader;
use quick_xml::events::Event;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};
use thiserror::Error;
use url::Url;

use crate::storage::models::{Priority, Status, Task};

#[derive(Debug, Error)]
pub enum CalDavError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Invalid header value")]
    Header,
    #[error("Server returned status {0}")]
    Status(u16),
    #[error("XML parse error: {0}")]
    Xml(String),
    #[error("iCalendar parse error: {0}")]
    ICal(String),
    #[error("No principal discovered")]
    NoPrincipal,
    #[error("No calendar home discovered")]
    NoCalendarHome,
}

pub type Result<T> = std::result::Result<T, CalDavError>;

#[derive(Debug, Clone)]
pub struct CalDavClient {
    base_url: Url,
    auth_header: HeaderValue,
    http: Client,
}

#[derive(Debug, Clone)]
pub struct RemoteCalendar {
    pub url: Url,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct RemoteTodo {
    pub href: Url,
    pub etag: Option<String>,
    pub ical: String,
}

impl CalDavClient {
    pub fn new(base_url: &str, username: &str, password: &str) -> Result<Self> {
        let base_url = Url::parse(base_url)?;
        let token =
            base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
        let auth_header =
            HeaderValue::from_str(&format!("Basic {token}")).map_err(|_| CalDavError::Header)?;
        let http = Client::builder()
            .user_agent("cosmic-tasks-caldav/0.1")
            .build()?;
        Ok(Self {
            base_url,
            auth_header,
            http,
        })
    }

    fn headers(&self, depth: Option<&str>, content_type: Option<&str>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("authorization"),
            self.auth_header.clone(),
        );
        if let Some(d) = depth {
            headers.insert(
                HeaderName::from_static("depth"),
                HeaderValue::from_str(d).map_err(|_| CalDavError::Header)?,
            );
        }
        if let Some(ct) = content_type {
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_str(ct).map_err(|_| CalDavError::Header)?,
            );
        }
        Ok(headers)
    }

    async fn request(
        &self,
        method: Method,
        url: Url,
        headers: HeaderMap,
        body: Option<String>,
    ) -> Result<(reqwest::StatusCode, HeaderMap, String)> {
        let mut req = self.http.request(method, url).headers(headers);
        if let Some(b) = body {
            req = req.body(b);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let text = resp.text().await?;
        Ok((status, headers, text))
    }

    /// Verifies credentials by issuing a PROPFIND on the base URL.
    pub async fn test_connection(&self) -> Result<()> {
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:current-user-principal/>
  </d:prop>
</d:propfind>"#
            .to_string();
        let headers = self.headers(Some("0"), Some("application/xml; charset=utf-8"))?;
        let method = Method::from_bytes(b"PROPFIND").unwrap();
        let (status, _, _) = self
            .request(method, self.base_url.clone(), headers, Some(body))
            .await?;
        if status.is_success() || status.as_u16() == 207 {
            Ok(())
        } else {
            Err(CalDavError::Status(status.as_u16()))
        }
    }

    async fn discover_principal(&self) -> Result<Url> {
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop><d:current-user-principal/></d:prop>
</d:propfind>"#
            .to_string();
        let headers = self.headers(Some("0"), Some("application/xml; charset=utf-8"))?;
        let method = Method::from_bytes(b"PROPFIND").unwrap();
        let (status, _, text) = self
            .request(method, self.base_url.clone(), headers, Some(body))
            .await?;
        if !(status.is_success() || status.as_u16() == 207) {
            return Err(CalDavError::Status(status.as_u16()));
        }
        let href =
            first_inner_href(&text, "current-user-principal").ok_or(CalDavError::NoPrincipal)?;
        Ok(self.base_url.join(&href)?)
    }

    async fn discover_calendar_home(&self, principal: &Url) -> Result<Url> {
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop><c:calendar-home-set/></d:prop>
</d:propfind>"#
            .to_string();
        let headers = self.headers(Some("0"), Some("application/xml; charset=utf-8"))?;
        let method = Method::from_bytes(b"PROPFIND").unwrap();
        let (status, _, text) = self
            .request(method, principal.clone(), headers, Some(body))
            .await?;
        if !(status.is_success() || status.as_u16() == 207) {
            return Err(CalDavError::Status(status.as_u16()));
        }
        let href =
            first_inner_href(&text, "calendar-home-set").ok_or(CalDavError::NoCalendarHome)?;
        Ok(self.base_url.join(&href)?)
    }

    /// Returns calendars under the user's home that advertise VTODO support.
    pub async fn list_task_calendars(&self) -> Result<Vec<RemoteCalendar>> {
        let principal = self.discover_principal().await?;
        let home = self.discover_calendar_home(&principal).await?;
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:displayname/>
    <d:resourcetype/>
    <c:supported-calendar-component-set/>
  </d:prop>
</d:propfind>"#
            .to_string();
        let headers = self.headers(Some("1"), Some("application/xml; charset=utf-8"))?;
        let method = Method::from_bytes(b"PROPFIND").unwrap();
        let (status, _, text) = self
            .request(method, home.clone(), headers, Some(body))
            .await?;
        if !(status.is_success() || status.as_u16() == 207) {
            return Err(CalDavError::Status(status.as_u16()));
        }
        let responses = parse_multistatus(&text)?;
        let mut out = vec![];
        for r in responses {
            if !r.is_collection_calendar {
                continue;
            }
            if !r.supports_vtodo {
                continue;
            }
            let mut url = self.base_url.join(&r.href)?;
            // Skip the home itself if it appeared in the listing.
            if url == home {
                continue;
            }
            // CalDAV collections are always directories; ensure the trailing
            // slash so `Url::join("uid.ics")` appends rather than replacing
            // the last segment.
            if !url.path().ends_with('/') {
                url.set_path(&format!("{}/", url.path()));
            }
            let display_name = r.display_name.unwrap_or_else(|| {
                url.path_segments()
                    .and_then(|s| {
                        s.filter(|x| !x.is_empty())
                            .next_back()
                            .map(|x| x.to_string())
                    })
                    .unwrap_or_else(|| "Calendar".to_string())
            });
            out.push(RemoteCalendar { url, display_name });
        }
        Ok(out)
    }

    pub async fn fetch_todos(&self, calendar: &Url) -> Result<Vec<RemoteTodo>> {
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:getetag/>
    <c:calendar-data/>
  </d:prop>
  <c:filter>
    <c:comp-filter name="VCALENDAR">
      <c:comp-filter name="VTODO"/>
    </c:comp-filter>
  </c:filter>
</c:calendar-query>"#
            .to_string();
        let headers = self.headers(Some("1"), Some("application/xml; charset=utf-8"))?;
        let method = Method::from_bytes(b"REPORT").unwrap();
        let (status, _, text) = self
            .request(method, calendar.clone(), headers, Some(body))
            .await?;
        if !(status.is_success() || status.as_u16() == 207) {
            return Err(CalDavError::Status(status.as_u16()));
        }
        let responses = parse_multistatus(&text)?;
        let mut out = vec![];
        for r in responses {
            let Some(ical) = r.calendar_data else {
                continue;
            };
            let href = self.base_url.join(&r.href)?;
            out.push(RemoteTodo {
                href,
                etag: r.etag,
                ical,
            });
        }
        Ok(out)
    }

    pub async fn put_todo(
        &self,
        href: &Url,
        ical: &str,
        if_match: Option<&str>,
    ) -> Result<Option<String>> {
        let mut headers = self.headers(None, Some("text/calendar; charset=utf-8"))?;
        if let Some(etag) = if_match {
            headers.insert(
                HeaderName::from_static("if-match"),
                HeaderValue::from_str(etag).map_err(|_| CalDavError::Header)?,
            );
        } else {
            headers.insert(
                HeaderName::from_static("if-none-match"),
                HeaderValue::from_static("*"),
            );
        }
        let (status, resp_headers, body) = self
            .request(Method::PUT, href.clone(), headers, Some(ical.to_string()))
            .await?;
        if !status.is_success() {
            tracing::warn!("PUT {href} -> {} body: {body}", status.as_u16());
            return Err(CalDavError::Status(status.as_u16()));
        }
        Ok(resp_headers
            .get("etag")
            .and_then(|v| v.to_str().ok().map(|s| s.to_string())))
    }

    #[allow(dead_code)]
    pub async fn delete_todo(&self, href: &Url, if_match: Option<&str>) -> Result<()> {
        let mut headers = self.headers(None, None)?;
        if let Some(etag) = if_match {
            headers.insert(
                HeaderName::from_static("if-match"),
                HeaderValue::from_str(etag).map_err(|_| CalDavError::Header)?,
            );
        }
        let (status, _, _) = self
            .request(Method::DELETE, href.clone(), headers, None)
            .await?;
        if !status.is_success() && status.as_u16() != 404 {
            return Err(CalDavError::Status(status.as_u16()));
        }
        Ok(())
    }
}

// --- minimal XML helpers ----------------------------------------------------

#[derive(Default, Debug)]
struct DavResponse {
    href: String,
    display_name: Option<String>,
    etag: Option<String>,
    calendar_data: Option<String>,
    is_collection_calendar: bool,
    supports_vtodo: bool,
}

fn local_name(name: &[u8]) -> &[u8] {
    match name.iter().rposition(|b| *b == b':') {
        Some(i) => &name[i + 1..],
        None => name,
    }
}

fn append_target(c: &mut DavResponse, target: &str, s: &str) {
    match target {
        "href" => {
            if c.href.is_empty() {
                c.href = s.to_string();
            }
        }
        "displayname" => match &mut c.display_name {
            Some(existing) => existing.push_str(s),
            None => c.display_name = Some(s.to_string()),
        },
        "etag" => match &mut c.etag {
            Some(existing) => existing.push_str(s),
            None => c.etag = Some(s.to_string()),
        },
        "caldata" => match &mut c.calendar_data {
            Some(existing) => existing.push_str(s),
            None => c.calendar_data = Some(s.to_string()),
        },
        _ => {}
    }
}

fn parse_multistatus(xml: &str) -> Result<Vec<DavResponse>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut out = vec![];
    let mut buf = vec![];
    let mut stack: Vec<Vec<u8>> = vec![];
    let mut current: Option<DavResponse> = None;
    let mut text_target: Option<&'static str> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(CalDavError::Xml(e.to_string())),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let local = local_name(e.name().as_ref()).to_vec();
                stack.push(local.clone());
                match local.as_slice() {
                    b"response" => current = Some(DavResponse::default()),
                    b"href" => {
                        // Only top-level <href> directly under <response> is the resource href.
                        // Nested hrefs (inside current-user-principal etc.) are handled by callers.
                        text_target = Some("href");
                    }
                    b"displayname" => text_target = Some("displayname"),
                    b"getetag" => text_target = Some("etag"),
                    b"calendar-data" => text_target = Some("caldata"),
                    b"calendar" => {
                        if let Some(c) = current.as_mut() {
                            if stack.iter().any(|n| n == b"resourcetype") {
                                c.is_collection_calendar = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                let local = local_name(e.name().as_ref()).to_vec();
                match local.as_slice() {
                    b"calendar" => {
                        if let Some(c) = current.as_mut() {
                            if stack.iter().any(|n| n == b"resourcetype") {
                                c.is_collection_calendar = true;
                            }
                        }
                    }
                    b"comp" => {
                        // <c:comp name="VTODO"/> inside supported-calendar-component-set
                        if let Some(c) = current.as_mut() {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"name"
                                    && attr.value.as_ref().eq_ignore_ascii_case(b"VTODO")
                                {
                                    c.supports_vtodo = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                if let (Some(target), Some(c)) = (text_target, current.as_mut()) {
                    let s = t.unescape().map_err(|e| CalDavError::Xml(e.to_string()))?;
                    append_target(c, target, s.as_ref());
                }
            }
            Ok(Event::CData(t)) => {
                if let (Some(target), Some(c)) = (text_target, current.as_mut()) {
                    let bytes = t.into_inner();
                    let s = std::str::from_utf8(&bytes)
                        .map_err(|e| CalDavError::Xml(e.to_string()))?
                        .to_string();
                    append_target(c, target, &s);
                }
            }
            Ok(Event::End(e)) => {
                let local = local_name(e.name().as_ref()).to_vec();
                if !stack.is_empty() {
                    stack.pop();
                }
                if local == b"response" {
                    if let Some(c) = current.take() {
                        out.push(c);
                    }
                }
                text_target = None;
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

/// Extract the first <href> nested under a named element (e.g. "current-user-principal").
fn first_inner_href(xml: &str, parent_local: &str) -> Option<String> {
    let parent_bytes = parent_local.as_bytes();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = vec![];
    let mut depth_in_parent: i32 = 0;
    let mut want_text = false;
    let mut found: Option<String> = None;
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => return None,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let local = local_name(e.name().as_ref()).to_vec();
                if local == parent_bytes {
                    depth_in_parent += 1;
                } else if depth_in_parent > 0 && local == b"href" {
                    want_text = true;
                }
            }
            Ok(Event::Text(t)) => {
                if want_text && depth_in_parent > 0 && found.is_none() {
                    if let Ok(s) = t.unescape() {
                        found = Some(s.into_owned());
                    }
                }
                want_text = false;
            }
            Ok(Event::End(e)) => {
                let local = local_name(e.name().as_ref()).to_vec();
                if local == parent_bytes {
                    depth_in_parent -= 1;
                }
                want_text = false;
            }
            _ => {}
        }
        buf.clear();
    }
    found
}

// --- VTODO <-> Task mapping -------------------------------------------------

pub fn parse_vtodo(ical: &str) -> std::result::Result<Todo, CalDavError> {
    let cal: ICalendar = ical.parse().map_err(|e: String| CalDavError::ICal(e))?;
    cal.components
        .into_iter()
        .find_map(|c| match c {
            icalendar::CalendarComponent::Todo(t) => Some(t),
            _ => None,
        })
        .ok_or_else(|| CalDavError::ICal("no VTODO in iCalendar object".into()))
}

pub fn vtodo_to_task(todo: &Todo, list_path: std::path::PathBuf) -> Task {
    let now = Utc::now();
    let uid = todo.get_uid().unwrap_or("").to_string();
    let summary = todo.get_summary().unwrap_or("").to_string();
    let description = todo.get_description().unwrap_or("").to_string();

    let status = match todo.property_value("STATUS") {
        Some("COMPLETED") => Status::Completed,
        _ => Status::NotStarted,
    };

    let priority = match todo
        .property_value("PRIORITY")
        .and_then(|s| s.parse::<u8>().ok())
    {
        Some(0) => Priority::Low,
        Some(p) if p <= 4 => Priority::High,
        Some(p) if p <= 6 => Priority::Normal,
        Some(_) => Priority::Low,
        None => Priority::Low,
    };

    // DUE accepts every variant the icalendar crate understands: DATE,
    // DATE-TIME UTC (`...Z`), floating DATE-TIME, and DATE-TIME with a TZID
    // parameter. Falls back to a textual parse for the loose forms some
    // servers emit (e.g. ISO-8601 with separators).
    let due_date = todo
        .get_due()
        .map(date_perhaps_time_to_utc)
        .or_else(|| todo.property_value("DUE").and_then(parse_ical_datetime));
    let completion_date = todo.get_completed().or_else(|| {
        todo.property_value("COMPLETED")
            .and_then(parse_ical_datetime)
    });
    let created = todo
        .property_value("CREATED")
        .and_then(parse_ical_datetime)
        .or_else(|| todo.property_value("DTSTAMP").and_then(parse_ical_datetime))
        .unwrap_or(now);
    let last_modified = todo
        .property_value("LAST-MODIFIED")
        .and_then(parse_ical_datetime)
        .or_else(|| todo.property_value("DTSTAMP").and_then(parse_ical_datetime))
        .unwrap_or(created);
    let tags = todo
        .property_value("CATEGORIES")
        .map(|s| {
            s.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();

    Task {
        id: uid,
        path: list_path,
        title: summary,
        favorite: false,
        today: false,
        status,
        priority,
        tags,
        notes: description,
        completion_date,
        due_date,
        reminder_date: None,
        recurrence: Default::default(),
        expanded: false,
        sub_tasks: vec![],
        deletion_date: None,
        created_date_time: created,
        last_modified_date_time: last_modified,
    }
}

pub fn task_to_vtodo(task: &Task) -> String {
    let mut todo = Todo::new();
    todo.uid(&task.id);
    todo.summary(&task.title);
    if !task.notes.is_empty() {
        todo.description(&task.notes);
    }
    todo.add_property(
        "STATUS",
        match task.status {
            Status::Completed => "COMPLETED",
            Status::NotStarted => "NEEDS-ACTION",
        },
    );
    let prio = match task.priority {
        Priority::High => "1",
        Priority::Normal => "5",
        Priority::Low => "9",
    };
    todo.add_property("PRIORITY", prio);
    if let Some(due) = task.due_date {
        // The UI only picks dates (no time-of-day), and SetDueDate stores a
        // local-midnight value. Detect that and emit VALUE=DATE so other
        // CalDAV clients show the same calendar day regardless of timezone.
        if is_local_date_only(due) {
            todo.due(due.with_timezone(&chrono::Local).date_naive());
        } else {
            todo.due(due);
        }
    }
    if let Some(c) = task.completion_date {
        todo.completed(c);
    }
    if !task.tags.is_empty() {
        todo.add_property("CATEGORIES", task.tags.join(","));
    }
    todo.add_property("CREATED", format_ical_datetime(task.created_date_time));
    todo.add_property(
        "LAST-MODIFIED",
        format_ical_datetime(task.last_modified_date_time),
    );
    todo.add_property("DTSTAMP", format_ical_datetime(Utc::now()));

    let mut cal = ICalendar::new();
    cal.push(todo.done());
    cal.to_string()
}

/// True if `dt`, viewed in the user's local timezone, falls exactly on
/// midnight — the encoding the date picker emits for an all-day due date.
fn is_local_date_only(dt: DateTime<Utc>) -> bool {
    use chrono::Timelike;
    let local = dt.with_timezone(&chrono::Local);
    local.hour() == 0 && local.minute() == 0 && local.second() == 0 && local.nanosecond() == 0
}

/// Reduce any iCalendar date/date-time variant into a UTC instant suitable
/// for the local Task model. Floating and TZID-bearing times are taken at
/// face value (chrono-tz is not enabled, so TZID can't be resolved).
fn date_perhaps_time_to_utc(dpt: DatePerhapsTime) -> DateTime<Utc> {
    match dpt {
        DatePerhapsTime::Date(d) => date_at_midnight_utc(d),
        DatePerhapsTime::DateTime(CalendarDateTime::Utc(dt)) => dt,
        DatePerhapsTime::DateTime(CalendarDateTime::Floating(naive)) => {
            DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
        }
        DatePerhapsTime::DateTime(CalendarDateTime::WithTimezone { date_time, .. }) => {
            DateTime::<Utc>::from_naive_utc_and_offset(date_time, Utc)
        }
    }
}

fn date_at_midnight_utc(d: NaiveDate) -> DateTime<Utc> {
    DateTime::<Utc>::from_naive_utc_and_offset(d.and_hms_opt(0, 0, 0).unwrap_or_default(), Utc)
}

/// Loose textual parser used as a fallback when the icalendar crate refuses
/// the input. Accepts:
///   - `20260101T120000Z` / `20260101T120000`        (basic iCal)
///   - `20260101`                                    (DATE)
///   - `2026-01-01T12:00:00Z` / `2026-01-01T12:00:00` (extended ISO-8601)
///   - `2026-01-01`                                  (extended date)
///   - `2026-01-01T12:00:00+02:00` (with offset)
fn parse_ical_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // RFC 3339 / ISO-8601 with offset.
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }

    let no_z = s.strip_suffix('Z').unwrap_or(s);

    // Basic and extended date-time forms.
    for fmt in ["%Y%m%dT%H%M%S", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S"] {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(no_z, fmt) {
            return Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
        }
    }

    // Date-only forms.
    for fmt in ["%Y%m%d", "%Y-%m-%d"] {
        if let Ok(date) = NaiveDate::parse_from_str(no_z, fmt) {
            return Some(date_at_midnight_utc(date));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zulu_datetime() {
        let dt = parse_ical_datetime("20260405T170617Z").expect("should parse");
        assert_eq!(
            dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "2026-04-05T17:06:17Z"
        );
    }

    #[test]
    fn parses_floating_datetime() {
        assert!(parse_ical_datetime("20260101T120000").is_some());
    }

    #[test]
    fn parses_date_only() {
        assert!(parse_ical_datetime("20260330").is_some());
    }

    #[test]
    fn parses_iso8601_extended_date_time() {
        let dt = parse_ical_datetime("2026-04-05T17:06:17Z").expect("iso-8601 utc");
        assert_eq!(
            dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "2026-04-05T17:06:17Z"
        );
    }

    #[test]
    fn parses_iso8601_with_offset() {
        let dt = parse_ical_datetime("2026-04-05T19:06:17+02:00").expect("iso-8601 offset");
        assert_eq!(
            dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "2026-04-05T17:06:17Z"
        );
    }

    #[test]
    fn parses_iso8601_extended_date_only() {
        assert!(parse_ical_datetime("2026-04-05").is_some());
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_ical_datetime("not a date").is_none());
        assert!(parse_ical_datetime("").is_none());
    }

    #[test]
    fn date_only_round_trip_emits_value_date() {
        use chrono::{Local, TimeZone};
        // Construct a local-midnight value, the same way the date picker does.
        let local_midnight = Local
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2026, 4, 28)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
            )
            .single()
            .unwrap()
            .with_timezone(&Utc);
        let task = Task {
            id: "abc".into(),
            path: std::path::PathBuf::from("/tmp"),
            title: "t".into(),
            due_date: Some(local_midnight),
            created_date_time: Utc::now(),
            last_modified_date_time: Utc::now(),
            ..Default::default()
        };
        let ical = task_to_vtodo(&task);
        assert!(
            ical.contains("DUE;VALUE=DATE:20260428"),
            "expected DUE as VALUE=DATE for local 2026-04-28; got:\n{ical}"
        );
    }
}

fn format_ical_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y%m%dT%H%M%SZ").to_string()
}
