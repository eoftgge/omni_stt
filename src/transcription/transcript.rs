use crate::transcription::subtitles::SubtitleBlock;
use std::io::Write;
use std::sync::OnceLock;
use time::{OffsetDateTime, UtcOffset};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};

const DIRECTORY: &str = "transcripts";
const FILE_NAME: &str = "omni.txt";

/// `Some(None)` means the offset could not be determined and everything falls
/// back to UTC; still empty means `init_local_offset` was never called.
static LOCAL_OFFSET: OnceLock<Option<UtcOffset>> = OnceLock::new();

/// Reads the machine's UTC offset once, and has to be called from `main`
/// before a single thread is spawned.
///
/// On Unix `time` refuses to read the local offset from a process that already
/// has more than one thread: another thread could be inside `setenv` at that
/// moment, and reading the timezone is not safe against that. Both
/// `tokio::runtime::Runtime::new` and the log appender spawn threads, so the
/// top of `main` is the only place left where this still succeeds.
///
/// Moving the call later does not fail loudly — it simply starts returning
/// `Err`, and every timestamp in the transcript silently becomes UTC.
pub fn init_local_offset() {
    let _ = LOCAL_OFFSET.set(UtcOffset::current_local_offset().ok());
}

/// False when the offset could not be read, so timestamps will be in UTC.
pub fn is_local_offset_known() -> bool {
    matches!(LOCAL_OFFSET.get(), Some(Some(_)))
}

/// Wall-clock time in the offset captured at startup, or UTC if there was none.
fn now() -> OffsetDateTime {
    let offset = LOCAL_OFFSET
        .get()
        .copied()
        .flatten()
        .unwrap_or(UtcOffset::UTC);

    OffsetDateTime::now_utc().to_offset(offset)
}

pub struct TranscriptWriter {
    sink: NonBlocking,
    _guard: WorkerGuard,
}

impl TranscriptWriter {
    pub fn open() -> Self {
        let appender = tracing_appender::rolling::never(DIRECTORY, FILE_NAME);
        let (sink, guard) = tracing_appender::non_blocking(appender);
        let mut writer = Self {
            sink,
            _guard: guard,
        };

        let at = now();
        writer.put(&format!(
            "\n=== {:04}-{:02}-{:02} {:02}:{:02} ===\n",
            at.year(),
            u8::from(at.month()),
            at.day(),
            at.hour(),
            at.minute(),
        ));

        writer
    }

    pub fn write(&mut self, block: &SubtitleBlock) {
        let at = now();
        let stamp = format!("{:02}:{:02}:{:02}", at.hour(), at.minute(), at.second());

        let line = match &block.speaker {
            Some(speaker) => format!("{stamp}  [{speaker}] {}\n", block.text),
            None => format!("{stamp}  {}\n", block.text),
        };

        self.put(&line);
    }

    fn put(&mut self, line: &str) {
        if let Err(e) = self.sink.write_all(line.as_bytes()) {
            tracing::error!("Failed to write transcript: {e}");
        }
    }
}
