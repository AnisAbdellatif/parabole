use super::Window;
use super::time::parse_time;
use crate::model::Programme;
use anyhow::{Context, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use std::collections::HashSet;
use std::io::BufRead;

struct Pending {
    channel: String,
    start: i64,
    stop: i64,
    title: Option<String>,
    descr: Option<String>,
}

#[derive(PartialEq)]
enum Field {
    Title,
    Desc,
}

/// Streams XMLTV, keeping programmes of `channels` that overlap `window`.
pub fn parse(
    r: impl BufRead,
    channels: &HashSet<String>,
    window: Window,
) -> Result<Vec<Programme>> {
    let mut reader = Reader::from_reader(r);
    let mut buf = Vec::new();
    let mut out = Vec::new();
    let mut cur: Option<Pending> = None;
    let mut field: Option<(Field, String)> = None;
    loop {
        buf.clear();
        match reader
            .read_event_into(&mut buf)
            .context("invalid guide XML")?
        {
            Event::Eof => break,
            Event::Start(e) => match e.name().as_ref() {
                b"programme" => cur = open_programme(&e, channels, window)?,
                b"title" => start_field(&cur, &mut field, Field::Title, |p| p.title.is_none()),
                b"desc" => start_field(&cur, &mut field, Field::Desc, |p| p.descr.is_none()),
                _ => {}
            },
            Event::Text(t) => {
                if let Some((_, s)) = field.as_mut() {
                    s.push_str(&t.decode().context("invalid guide XML")?);
                }
            }
            Event::CData(t) => {
                if let Some((_, s)) = field.as_mut() {
                    s.push_str(&t.decode().context("invalid guide XML")?);
                }
            }
            Event::GeneralRef(r) => {
                if let Some((_, s)) = field.as_mut() {
                    let c = match r.resolve_char_ref().context("invalid guide XML")? {
                        Some(c) => Some(c),
                        None => predefined(&r.decode().context("invalid guide XML")?),
                    };
                    s.extend(c);
                }
            }
            Event::End(e) => match e.name().as_ref() {
                b"title" | b"desc" => {
                    if let (Some((f, s)), Some(p)) = (field.take(), cur.as_mut()) {
                        let s = s.trim().to_string();
                        match f {
                            Field::Title => p.title = Some(s),
                            Field::Desc => p.descr = (!s.is_empty()).then_some(s),
                        }
                    }
                }
                b"programme" => {
                    if let Some(p) = cur.take()
                        && let Some(title) = p.title.filter(|t| !t.is_empty())
                    {
                        out.push(Programme {
                            channel: p.channel,
                            start: p.start,
                            stop: p.stop,
                            title,
                            descr: p.descr,
                        });
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(out)
}

fn start_field(
    cur: &Option<Pending>,
    field: &mut Option<(Field, String)>,
    f: Field,
    first: impl Fn(&Pending) -> bool,
) {
    if cur.as_ref().is_some_and(first) {
        *field = Some((f, String::new()));
    }
}

fn predefined(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => None,
    }
}

fn open_programme(
    e: &BytesStart,
    channels: &HashSet<String>,
    window: Window,
) -> Result<Option<Pending>> {
    let (mut channel, mut start, mut stop) = (None, None, None);
    for a in e.attributes().flatten() {
        let v = a
            .normalized_value(XmlVersion::Implicit1_0)
            .context("invalid guide XML")?;
        match a.key.as_ref() {
            b"channel" => channel = Some(v.trim().to_lowercase()),
            b"start" => start = parse_time(&v),
            b"stop" => stop = parse_time(&v),
            _ => {}
        }
    }
    let (Some(channel), Some(start), Some(stop)) = (channel, start, stop) else {
        return Ok(None);
    };
    if !channels.contains(&channel) || stop <= start || stop <= window.from || start >= window.to {
        return Ok(None);
    }
    Ok(Some(Pending {
        channel,
        start,
        stop,
        title: None,
        descr: None,
    }))
}
