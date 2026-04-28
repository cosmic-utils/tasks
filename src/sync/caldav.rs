use base64::Engine as _;
use chrono::{DateTime, Utc};
use icalendar::{Calendar as ICalendar, Component, Todo};
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
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
        let token = base64::engine::general_purpose::STANDARD
            .encode(format!("{username}:{password}"));
        let auth_header = HeaderValue::from_str(&format!("Basic {token}"))
            .map_err(|_| CalDavError::Header)?;
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
        let href = first_inner_href(&text, "current-user-principal")
            .ok_or(CalDavError::NoPrincipal)?;
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
        let href = first_inner_href(&text, "calendar-home-set")
            .ok_or(CalDavError::NoCalendarHome)?;
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
            let url = self.base_url.join(&r.href)?;
            // Skip the home itself if it appeared in the listing.
            if url == home {
                continue;
            }
            let display_name = r.display_name.unwrap_or_else(|| {
                url.path_segments()
                    .and_then(|s| s.filter(|x| !x.is_empty()).last().map(|x| x.to_string()))
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
        let (status, resp_headers, _) = self
            .request(Method::PUT, href.clone(), headers, Some(ical.to_string()))
            .await?;
        if !status.is_success() {
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
                    b"collection" => {
                        // Inside <resourcetype>; combined with <calendar/> sibling means a calendar collection.
                        if let Some(c) = current.as_mut() {
                            // mark provisional; finalized when we also see <calendar/>
                            c.is_collection_calendar |= stack.iter().any(|n| n == b"resourcetype")
                                && false; // placeholder
                        }
                    }
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
    let cal: ICalendar = ical
        .parse()
        .map_err(|e: String| CalDavError::ICal(e))?;
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

    let due_date = todo
        .property_value("DUE")
        .and_then(parse_ical_datetime);
    let completion_date = todo
        .property_value("COMPLETED")
        .and_then(parse_ical_datetime);
    let created = todo
        .property_value("CREATED")
        .and_then(parse_ical_datetime)
        .unwrap_or(now);
    let last_modified = todo
        .property_value("LAST-MODIFIED")
        .and_then(parse_ical_datetime)
        .unwrap_or(now);
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
        todo.add_property("DUE", &format_ical_datetime(due));
    }
    if let Some(c) = task.completion_date {
        todo.add_property("COMPLETED", &format_ical_datetime(c));
    }
    if !task.tags.is_empty() {
        todo.add_property("CATEGORIES", &task.tags.join(","));
    }
    todo.add_property("CREATED", &format_ical_datetime(task.created_date_time));
    todo.add_property(
        "LAST-MODIFIED",
        &format_ical_datetime(task.last_modified_date_time),
    );

    let mut cal = ICalendar::new();
    cal.push(todo.done());
    cal.to_string()
}

fn parse_ical_datetime(s: &str) -> Option<DateTime<Utc>> {
    // Accept basic forms: 20260101T120000Z, 20260101T120000, 20260101.
    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ") {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S") {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y%m%d") {
        let naive = date.and_hms_opt(0, 0, 0)?;
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }
    None
}

fn format_ical_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y%m%dT%H%M%SZ").to_string()
}
