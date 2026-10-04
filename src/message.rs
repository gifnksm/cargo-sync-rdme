use std::{borrow::Cow, fmt, io::Write as _, sync::OnceLock};

use clap_verbosity_flag::VerbosityFilter;
use console::Color;

use crate::terminal::{StreamInfo, Terminal};

pub(crate) trait Message {
    fn to_str(&self) -> Cow<'_, str>;
}

impl Message for fmt::Arguments<'_> {
    fn to_str(&self) -> Cow<'_, str> {
        Cow::Owned(self.to_string())
    }
}

impl Message for str {
    fn to_str(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
}

static STREAM: OnceLock<StreamInfo> = OnceLock::new();
static OUTPUT_LEVEL: OnceLock<Level> = OnceLock::new();

pub(crate) fn init(terminal: &Terminal, filter: VerbosityFilter) {
    let stream = terminal.message_stream();
    STREAM.set(stream).unwrap();
    OUTPUT_LEVEL.set(filter.into()).unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<VerbosityFilter> for Level {
    fn from(value: VerbosityFilter) -> Self {
        match value {
            VerbosityFilter::Off => Self::Off,
            VerbosityFilter::Error => Self::Error,
            VerbosityFilter::Warn => Self::Warn,
            VerbosityFilter::Info => Self::Info,
            VerbosityFilter::Debug => Self::Debug,
            VerbosityFilter::Trace => Self::Trace,
        }
    }
}

impl Level {
    const fn prefix_str(self) -> &'static str {
        match self {
            Level::Off => unreachable!(),
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }

    const fn prefix_color(self) -> Color {
        match self {
            Level::Off => unreachable!(),
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::Green,
            Level::Debug => Color::Blue,
            Level::Trace => Color::Magenta,
        }
    }
}

const MAX_PREFIX_WIDTH: usize = 5;

fn print_message<M>(level: Level, message: &M)
where
    M: Message,
{
    assert!(level > Level::Off);
    let output_level = OUTPUT_LEVEL.get().copied().unwrap_or(Level::Off);
    if level > output_level {
        return;
    }

    let stream = STREAM.get().unwrap();
    let mut output = stream.lock();
    let styler = stream.styler();

    let head_prefix = level.prefix_str();
    let head_prefix = format_args!("{head_prefix:>MAX_PREFIX_WIDTH$}");
    let head_prefix = styler.styled(head_prefix).fg(level.prefix_color());

    let tail_prefix = format_args!("{:>MAX_PREFIX_WIDTH$}", "");

    let message = message.to_str();
    let mut message = message.lines();
    if let Some(line) = message.next() {
        let _ = writeln!(&mut output, "{head_prefix} {line}");
    }
    for line in message {
        let _ = writeln!(&mut output, "{tail_prefix} {line}");
    }
}

#[expect(dead_code)]
pub(crate) fn error<M>(message: &M)
where
    M: Message,
{
    print_message(Level::Error, message);
}

pub(crate) fn warn<M>(message: &M)
where
    M: Message,
{
    print_message(Level::Warn, message);
}

pub(crate) fn info<M>(message: &M)
where
    M: Message,
{
    print_message(Level::Info, message);
}

pub(crate) fn debug<M>(message: &M)
where
    M: Message,
{
    print_message(Level::Debug, message);
}

pub(crate) fn trace<M>(message: &M)
where
    M: Message,
{
    print_message(Level::Trace, message);
}

macro_rules! _error {
    ($($arg:tt)*) => {
        $crate::message::error(&format_args!($($arg)*));
    }
}
#[expect(unused_imports)]
pub(crate) use _error as error;

macro_rules! _warn {
    ($($arg:tt)*) => {
        $crate::message::warn(&format_args!($($arg)*));
    }
}
pub(crate) use _warn as warn;

macro_rules! _info {
    ($($arg:tt)*) => {
        $crate::message::info(&format_args!($($arg)*));
    }
}
pub(crate) use _info as info;

macro_rules! _debug {
    ($($arg:tt)*) => {
        $crate::message::debug(&format_args!($($arg)*));
    }
}
pub(crate) use _debug as debug;

macro_rules! _trace {
    ($($arg:tt)*) => {
        $crate::message::trace(&format_args!($($arg)*));
    }
}
pub(crate) use _trace as trace;
