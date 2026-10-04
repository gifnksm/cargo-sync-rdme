use std::io;

use clap::ColorChoice;
use supports_color::Stream;

#[derive(Debug)]
pub(crate) struct Terminal {
    stdout: StreamInfo,
    stderr: StreamInfo,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StreamInfo {
    kind: Stream,
    should_use_color: bool,
}

impl Terminal {
    pub(crate) fn init(choice: ColorChoice) -> Self {
        let stdout = StreamInfo::new(Stream::Stdout, choice);
        let stderr = StreamInfo::new(Stream::Stderr, choice);
        console::set_colors_enabled(stdout.should_use_color);
        console::set_colors_enabled_stderr(stderr.should_use_color);
        Self { stdout, stderr }
    }

    pub(crate) fn message_stream(&self) -> StreamInfo {
        self.stderr
    }

    pub(crate) fn diagnostic_stream(&self) -> StreamInfo {
        self.stderr
    }

    pub(crate) fn log_stream(&self) -> StreamInfo {
        self.stderr
    }

    pub(crate) fn output_stream(&self) -> StreamInfo {
        self.stdout
    }
}

impl StreamInfo {
    pub(crate) fn new(kind: Stream, choice: ColorChoice) -> Self {
        let should_use_color = match choice {
            ColorChoice::Always => true,
            ColorChoice::Auto => supports_color::on(kind).is_some(),
            ColorChoice::Never => false,
        };
        Self {
            kind,
            should_use_color,
        }
    }

    pub(crate) fn kind(self) -> Stream {
        self.kind
    }

    pub(crate) fn should_use_color(self) -> bool {
        self.should_use_color
    }

    pub(crate) fn styler(self) -> TerminalStyler {
        TerminalStyler::new(self)
    }

    pub(crate) fn lock(self) -> LockedStream<'static> {
        match self.kind {
            Stream::Stdout => LockedStream::Stdout(io::stdout().lock()),
            Stream::Stderr => LockedStream::Stderr(io::stderr().lock()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum LockedStream<'a> {
    Stdout(io::StdoutLock<'a>),
    Stderr(io::StderrLock<'a>),
}

impl io::Write for LockedStream<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            LockedStream::Stdout(s) => s.write(buf),
            LockedStream::Stderr(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            LockedStream::Stdout(s) => s.flush(),
            LockedStream::Stderr(s) => s.flush(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct TerminalStyler {
    stream: StreamInfo,
}

impl TerminalStyler {
    fn new(stream: StreamInfo) -> Self {
        Self { stream }
    }

    pub(crate) fn style(&self) -> console::Style {
        let s = console::Style::new();
        match self.stream.kind() {
            Stream::Stdout => s.for_stdout(),
            Stream::Stderr => s.for_stderr(),
        }
    }

    pub(crate) fn styled<D>(&self, val: D) -> console::StyledObject<D> {
        let s = console::style(val);
        match self.stream.kind() {
            Stream::Stdout => s.for_stdout(),
            Stream::Stderr => s.for_stderr(),
        }
    }
}
