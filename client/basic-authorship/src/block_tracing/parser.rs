//! `TraceEvent` parser
use super::*;

use nom::{
    IResult,
    branch::alt,
    bytes::{self, complete::tag},
    character::{self, complete::char},
    combinator::map,
    sequence::{delimited, preceded, tuple},
};

struct OptVal(Option<Vec<u8>>);

#[derive(Debug,PartialEq)]
struct Message {
    id: String,
    event: Event,
}

fn parse_message(message: impl AsRef<str>) -> Option<Message> {
    let id = bytes::complete::is_not(":");
    let left = preceded(
        character::complete::char(':'),
        preceded(
            character::complete::multispace0,
            map(bytes::complete::take_till1(|_| false), parse_event)
        )
    );

    let parsed: IResult<_, (&str, _)> =
        tuple((id, left))(message.as_ref());

    parsed.map(|(_, (id, event))| {
        event.map(|(_, event)| {
            Message {
                id: id.to_owned(),
                event,
            }
        }).ok()
    }).ok().flatten()
}

fn parse_event(input: &str) -> IResult<&str, Event> {
    alt((
        parse_put_child,
        parse_kill_child,
        parse_clear_prefix,
        parse_clear_child_prefix,
        parse_append,
        parse_not_concerned,
    ))(input)
}

/// event parse for `Event::PutChild`
fn parse_put_child(input: &str) -> IResult<&str, Event> {
    let arg = preceded(
        tag("PutChild"),
        delimited(
            char('('),
            bytes::complete::is_not(")"),
            char(')'),
        )
    );

    tuple(
        (arg, character::complete::multispace1, parse_k_equ_opt_v)
    )(input).map(|(left, (arg, _, (key, value)))| {
        (left, Event::PutChild(PutChild {
            child_id: arg.as_bytes().to_vec(),
            key: key.as_bytes().to_vec(),
            value: value.0,
        }))
    })
}

/// event parse for `Event::KillChild`
fn parse_kill_child(input: &str) -> IResult<&str, Event> {
    preceded(
        tag("KillChild"),
        delimited(
            char('('),
            bytes::complete::is_not(")"),
            char(')'),
        )
    )(input).map(|(left, arg)| {
        (left, Event::KillChild(KillChild {
            child_id: arg.as_bytes().to_vec(),
        }))
    })
}

/// event parse for `Event::ClearPrefix`
fn parse_clear_prefix(input: &str) -> IResult<&str, Event> {
    tuple(
        (tag("ClearPrefix"), bytes::complete::take_till(|c: char| !c.is_whitespace()))
    )(input).map(|(left, (_, value))| {
        (left, Event::ClearPrefix(ClearPrefix {
            prefix: value.as_bytes().to_vec(),
        }))
    })
}

/// event parse for `Event::ClearChildPrefix`
fn parse_clear_child_prefix(input: &str) -> IResult<&str, Event> {
    let arg = preceded(
        tag("ClearChildPrefix"),
        delimited(
            char('('),
            bytes::complete::is_not(")"),
            char(')'),
        )
    );

    let value = bytes::complete::take_till(|c: char| !c.is_whitespace());

    tuple((arg, value))(input).map(|(left, (arg, value))| {
        (left, Event::ClearChildPrefix(ClearChildPrefix {
            child_id: arg.as_bytes().to_vec(),
            prefix: value.as_bytes().to_vec(),
        }))
    })
}

/// event parse for `Event::Append`
fn parse_append(input: &str) -> IResult<&str, Event> {
    tuple(
        (tag("Append"), map(bytes::complete::take_till(|c: char| !c.is_whitespace()), parse_k_equ_v))
    )(input).map(|(_, (left, kv))| {
        let (_, (key, value)) = kv.unwrap_or_default();
        (left, Event::Append(Append {
            key: key.as_bytes().to_vec(),
            append: value.as_bytes().to_vec(),
        }))
    })
}

/// event parse fallback to `Event::NotConcerned`
fn parse_not_concerned(input: &str) -> IResult<&str, Event> {
    Ok((input, Event::NotConcerned))
}

/// k-v parser, eg. 0000=1111
fn parse_k_equ_v(input: &str) -> IResult<&str, (&str, &str)> {
    let (value, (key, _)) = tuple(
        (bytes::complete::take_while1(|c: char| c != '='), tag("="))
    )(input)?;

    Ok((value, (key, value)))
}

/// k-opt-v parser, eg. 0000=Some(1111)
fn parse_k_equ_opt_v(input: &str) -> IResult<&str, (&str, OptVal)> {
    let (value, (key, _, opt)) = tuple(
        (bytes::complete::take_while1(|c: char| c != '='), tag("="), alt((parse_none, parse_some)))
    )(input)?;

    Ok((value, (key, opt)))
}

fn parse_opt_val(value: impl AsRef<str>) -> Option<Vec<u8>> {
    let parsed: IResult<_, OptVal> =
        alt((parse_none, parse_some))(value.as_ref());

    parsed.map(|r| r.1.0).ok().flatten()
}

/// eg. None
fn parse_none(input: &str) -> IResult<&str, OptVal> {
    map(tag("None"), |_: &str| OptVal(None))(input)
}

/// eg. Some(00000bbbbb)
fn parse_some(input: &str) -> IResult<&str, OptVal> {
    preceded(
        tag("Some"),
        delimited(
            char('('),
            map(bytes::complete::is_not(")"), |v: &str| {
                OptVal(Some(v.as_bytes().to_vec()))
            }),
            char(')'),
        ),
    )(input)
}

impl From<TraceEvent> for Event {
    fn from(event: TraceEvent) -> Self {
        use parser::*;
        let values = event.values.string_values;
        // Get & Put
        match values.get("method") {
            Some(value) => {
                match &value[..] {
                    "Put" => {
                        let key = values.get("key").unwrap();
                        let value = values.get("value").unwrap();

                        Self::Put(Put {
                            key: key.as_bytes().to_vec(),
                            value: parse_opt_val(value),
                        })
                    }

                    // NB: ignore other methods
                    _ => Self::NotConcerned,
                }
            }

            None => {
                // Other Event
                match values.get("message") {
                    Some(value) => {
                        parse_message(value).map(|message| message.event).unwrap_or(Self::NotConcerned)
                    }

                    None => {
                        Self::NotConcerned
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

    #[test]
    fn test_opt_val_parser() {
        let parsed = parse_opt_val("None");
        assert_eq!(parsed, None);
        let parsed = parse_opt_val("Some()");
        assert_eq!(parsed, None);
        let parsed = parse_opt_val("Some(0000abcd)");
        assert_eq!(parsed, Some(vec![b'0', b'0', b'0', b'0', b'a', b'b', b'c', b'd']));
    }

    #[test]
    fn test_message_parser() {
        let parsed = parse_message("0001: PutChild(0002) K=V");
        assert_eq!(parsed, Some(Message { id: "0001".to_owned(), event: Event::PutChild(PutChild {
            child_id: vec![b'0', b'0', b'0', b'2'],
            key: vec![b'k', b'e', b'y'],
            value: None,
        }) }));
    }
}
