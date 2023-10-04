use crate::errors::*;
use crate::utils::{take_anchored_lines, take_lines};
use std::{
    ops::{Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeTo},
    str::FromStr,
};

use log::{error, warn};
use mdbook::BookItem;
use once_cell::sync::Lazy;
use regex::{CaptureMatches, Captures, Regex};
use url::Url;

const MAX_LINK_NESTED_DEPTH: usize = 10;
const ESCAPE_CHAR: char = '\\';

pub(crate) struct WebInclude;

impl WebInclude {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl mdbook::preprocess::Preprocessor for WebInclude {
    fn name(&self) -> &str {
        "webinclude"
    }

    fn supports_renderer(&self, _renderer: &str) -> bool {
        true
    }

    fn run(
        &self,
        ctx: &mdbook::preprocess::PreprocessorContext,
        mut book: mdbook::book::Book,
    ) -> mdbook::errors::Result<mdbook::book::Book> {
        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *item {
                chapter.content = replace_all(
                    &chapter.content,
                    0,
                    match ctx
                        .config
                        .get_preprocessor("webinclude")
                        .map(|x| x.get("headers"))
                    {
                        Some(Some(toml::value::Value::Table(h))) => Some(h),
                        _ => None,
                    },
                );
            };
        });
        Ok(book)
    }
}
fn replace_all(s: &str, depth: usize, headers: Option<&toml::value::Table>) -> String {
    let mut previous_end_index = 0;
    let mut replaced = String::new();

    for link in find_links(s) {
        replaced.push_str(&s[previous_end_index..link.start_index]);

        match link.render_with_url(headers) {
            Ok(new_content) => {
                if depth < MAX_LINK_NESTED_DEPTH {
                    replaced.push_str(&replace_all(&new_content, depth + 1, headers));
                } else {
                    error!(
                        // TODO: provide path & url
                        "Stack depth exceeded. Check for cyclic includes",
                    );
                }
                previous_end_index = link.end_index;
            }
            Err(e) => {
                error!("Error updating \"{}\", {}", link.link_text, e);
                for cause in e.chain().skip(1) {
                    warn!("Caused By: {}", cause);
                }

                // This should make sure we include the raw `{{# ... }}` snippet
                // in the page content if there are any errors.
                previous_end_index = link.start_index;
            }
        }
    }

    replaced.push_str(&s[previous_end_index..]);
    replaced
}

#[derive(PartialEq, Debug, Clone)]
enum LinkType {
    Escaped,
    WebInclude(Url, RangeOrAnchor),
}

#[derive(PartialEq, Debug, Clone)]
enum RangeOrAnchor {
    Range(LineRange),
    Anchor(String),
}

#[allow(clippy::enum_variant_names)] // The prefix can't be removed, and is meant to mirror the contained type
#[derive(PartialEq, Debug, Clone)]
enum LineRange {
    Range(Range<usize>),
    RangeFrom(RangeFrom<usize>),
    RangeTo(RangeTo<usize>),
    RangeFull(RangeFull),
}

impl RangeBounds<usize> for LineRange {
    fn start_bound(&self) -> Bound<&usize> {
        match self {
            LineRange::Range(r) => r.start_bound(),
            LineRange::RangeFrom(r) => r.start_bound(),
            LineRange::RangeTo(r) => r.start_bound(),
            LineRange::RangeFull(r) => r.start_bound(),
        }
    }

    fn end_bound(&self) -> Bound<&usize> {
        match self {
            LineRange::Range(r) => r.end_bound(),
            LineRange::RangeFrom(r) => r.end_bound(),
            LineRange::RangeTo(r) => r.end_bound(),
            LineRange::RangeFull(r) => r.end_bound(),
        }
    }
}

impl From<Range<usize>> for LineRange {
    fn from(r: Range<usize>) -> LineRange {
        LineRange::Range(r)
    }
}

impl From<RangeFrom<usize>> for LineRange {
    fn from(r: RangeFrom<usize>) -> LineRange {
        LineRange::RangeFrom(r)
    }
}

impl From<RangeTo<usize>> for LineRange {
    fn from(r: RangeTo<usize>) -> LineRange {
        LineRange::RangeTo(r)
    }
}

impl From<RangeFull> for LineRange {
    fn from(r: RangeFull) -> LineRange {
        LineRange::RangeFull(r)
    }
}

fn parse_include_path(args: &str) -> LinkType {
    let (url, span) = args
        .split_once(' ')
        .map(|x| (x.0, Some(x.1)))
        .unwrap_or((args, None));

    let url: Url = Url::from_str(url).expect("invalid URL format");
    let range_or_anchor = parse_range_or_anchor(span);

    LinkType::WebInclude(url, range_or_anchor)
}

fn parse_range_or_anchor(parts: Option<&str>) -> RangeOrAnchor {
    let mut parts = parts.unwrap_or("").splitn(3, ':').fuse();

    let next_element = parts.next();
    let start = if let Some(value) = next_element.and_then(|s| s.parse::<usize>().ok()) {
        // subtract 1 since line numbers usually begin with 1
        Some(value.saturating_sub(1))
    } else if let Some("") = next_element {
        None
    } else if let Some(anchor) = next_element {
        return RangeOrAnchor::Anchor(String::from(anchor));
    } else {
        None
    };

    let end = parts.next();
    // If `end` is empty string or any other value that can't be parsed as a usize, treat this
    // include as a range with only a start bound. However, if end isn't specified, include only
    // the single line specified by `start`.
    let end = end.map(|s| s.parse::<usize>());

    match (start, end) {
        (Some(start), Some(Ok(end))) => RangeOrAnchor::Range(LineRange::from(start..end)),
        (Some(start), Some(Err(_))) => RangeOrAnchor::Range(LineRange::from(start..)),
        (Some(start), None) => RangeOrAnchor::Range(LineRange::from(start..start + 1)),
        (None, Some(Ok(end))) => RangeOrAnchor::Range(LineRange::from(..end)),
        (None, None) | (None, Some(Err(_))) => RangeOrAnchor::Range(LineRange::from(RangeFull)),
    }
}

#[derive(PartialEq, Debug, Clone)]
struct Link<'a> {
    start_index: usize,
    end_index: usize,
    link_type: LinkType,
    link_text: &'a str,
}

impl<'a> Link<'a> {
    fn from_capture(cap: Captures<'a>) -> Option<Link<'a>> {
        let link_type = match (cap.get(0), cap.get(1), cap.get(2)) {
            (_, Some(kind), Some(args)) if kind.as_str() == "webinclude" => {
                Some(parse_include_path(args.as_str()))
            }
            (Some(mat), None, None) if mat.as_str().starts_with(ESCAPE_CHAR) => {
                Some(LinkType::Escaped)
            }
            _ => None,
        };

        link_type.and_then(|lnk_type| {
            cap.get(0).map(|mat| Link {
                start_index: mat.start(),
                end_index: mat.end(),
                link_type: lnk_type,
                link_text: mat.as_str(),
            })
        })
    }

    fn render_with_url(&self, headers: Option<&toml::value::Table>) -> Result<String> {
        match self.link_type {
            // omit the escape char
            LinkType::Escaped => Ok(self.link_text[1..].to_owned()),
            LinkType::WebInclude(ref url, ref span) => {
                let mut req = ureq::get(url.as_str());
                if let Some(h) = headers {
                    for (key, value) in h {
                        if let toml::Value::String(s) = value {
                            req = req.set(key, s);
                        };
                    }
                };
                req.call()
                    .with_context(|| format!("Could not query URL {} ({})", url, self.link_text))
                    .map(|res| {
                        res.into_string().with_context(|| {
                            format!("Expected UTF-8 in {} ({})", url, self.link_text)
                        })
                    })
                    .and_then(|x| Ok(x?))
                    .map(|s| match span {
                        RangeOrAnchor::Range(range) => take_lines(&s, range.clone()),
                        RangeOrAnchor::Anchor(anchor) => take_anchored_lines(&s, anchor),
                    })
            }
        }
    }
}

struct LinkIter<'a>(CaptureMatches<'a, 'a>);

impl<'a> Iterator for LinkIter<'a> {
    type Item = Link<'a>;
    fn next(&mut self) -> Option<Link<'a>> {
        for cap in &mut self.0 {
            if let Some(inc) = Link::from_capture(cap) {
                return Some(inc);
            }
        }
        None
    }
}

fn find_links(contents: &str) -> LinkIter<'_> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?x)          # insignificant whitespace mode
        \\\{\{\#.*\}\}      # match escaped link
        |                   # or
        \{\{\s*             # link opening parens and whitespace
        \#([a-zA-Z0-9_]+)   # link type
        \s+                 # separating whitespace
        ([^}]+)             # link target path and space separated properties
        \}\}                # link closing parens",
        )
        .unwrap()
    });

    LinkIter(RE.captures_iter(contents))
}
