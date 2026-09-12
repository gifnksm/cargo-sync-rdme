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
}
