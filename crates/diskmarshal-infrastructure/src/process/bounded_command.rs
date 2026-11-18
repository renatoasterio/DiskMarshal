//! Running another program without ever waiting for it forever.

use std::io::{self, Read};
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// What to do with what the program prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stdout {
    /// Keep it, to read the answer of the program.
    Capture,
    /// Throw it away: only the exit code and the error text matter.
    Discard,
}

/// What a program left behind when it ended.
#[derive(Debug)]
pub(crate) struct CommandOutput {
    /// Exit code, or `None` when the program was ended by a signal.
    pub code: Option<i32>,
    /// What it printed, when that was captured.
    pub stdout: Vec<u8>,
    /// What it printed as errors.
    pub stderr: Vec<u8>,
}

impl CommandOutput {
    /// The first thing the program said went wrong, for an error message.
    pub(crate) fn first_error_line(&self) -> String {
        String::from_utf8_lossy(&self.stderr)
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("no message")
            .to_owned()
    }
}

/// Why a program produced no output.
#[derive(Debug)]
pub(crate) enum CommandFailure {
    /// The program could not be started.
    NotStarted(io::Error),
    /// The program was still running at the time limit and was stopped.
    TimedOut,
}

/// Runs `command` to its end, or stops it once `timeout` has passed.
///
/// The program gets no standard input. The caller names the program by an
/// absolute path and decides its arguments and environment.
pub(crate) fn run_bounded(
    command: &mut Command,
    stdout: Stdout,
    timeout: Duration,
) -> Result<CommandOutput, CommandFailure> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(match stdout {
            Stdout::Capture => Stdio::piped(),
            Stdout::Discard => Stdio::null(),
        })
        .stderr(Stdio::piped())
        .spawn()
        .map_err(CommandFailure::NotStarted)?;
    // Both streams are read while the program runs: a program that fills a
    // pipe nobody reads would never end.
    let printed = read_in_background(child.stdout.take());
    let errors = read_in_background(child.stderr.take());

    let Some(code) = wait(&mut child, timeout) else {
        // The readers are left behind on purpose. They end when the pipes
        // close, which a program started by the stopped one could delay.
        return Err(CommandFailure::TimedOut);
    };
    Ok(CommandOutput {
        code,
        stdout: collect(printed),
        stderr: collect(errors),
    })
}

/// Waits for the program up to the time limit. Returns its exit code, or
/// `None` when it had to be stopped.
fn wait(child: &mut Child, timeout: Duration) -> Option<Option<i32>> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status.code()),
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(None) | Err(_) => {
                // Stopping it is all that is left to do.
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

fn read_in_background<R: Read + Send + 'static>(stream: Option<R>) -> Option<JoinHandle<Vec<u8>>> {
    stream.map(|mut stream| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            // What could be read before a failure is still worth returning.
            let _ = stream.read_to_end(&mut bytes);
            bytes
        })
    })
}

fn collect(reader: Option<JoinHandle<Vec<u8>>>) -> Vec<u8> {
    reader
        .and_then(|reader| reader.join().ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::{Duration, Instant};

    use super::{CommandFailure, Stdout, run_bounded};

    const SHELL: &str = "/bin/sh";
    const GENEROUS: Duration = Duration::from_secs(30);

    fn shell(script: &str) -> Command {
        let mut command = Command::new(SHELL);
        command.args(["-c", script]).env_clear();
        command
    }

    #[test]
    fn returns_the_exit_code_and_both_streams() {
        let output = run_bounded(
            &mut shell("echo printed; echo failed >&2; exit 3"),
            Stdout::Capture,
            GENEROUS,
        )
        .unwrap();

        assert_eq!(output.code, Some(3));
        assert_eq!(output.stdout, b"printed\n");
        assert_eq!(output.stderr, b"failed\n");
    }

    #[test]
    fn the_first_error_line_skips_blank_lines() {
        let output = run_bounded(
            &mut shell("echo >&2; echo '  refused  ' >&2; echo later >&2"),
            Stdout::Discard,
            GENEROUS,
        )
        .unwrap();
        assert_eq!(output.first_error_line(), "refused");
        let silent = run_bounded(&mut shell("exit 1"), Stdout::Discard, GENEROUS).unwrap();
        assert_eq!(silent.first_error_line(), "no message");
    }

    #[test]
    fn discards_what_is_printed_when_asked_to() {
        let output = run_bounded(&mut shell("echo printed"), Stdout::Discard, GENEROUS).unwrap();
        assert_eq!(output.code, Some(0));
        assert!(output.stdout.is_empty());
    }

    #[test]
    fn a_program_that_prints_more_than_a_pipe_holds_still_ends() {
        let script = "i=0; while [ $i -lt 4000 ]; do echo 0123456789012345678901234567890123456789; i=$((i+1)); done";
        let output = run_bounded(&mut shell(script), Stdout::Capture, GENEROUS).unwrap();
        assert_eq!(output.stdout.len(), 4_000 * 41);
    }

    #[test]
    fn stops_a_program_that_takes_too_long() {
        let started = Instant::now();
        let result = run_bounded(
            &mut shell("exec /bin/sleep 30"),
            Stdout::Capture,
            Duration::from_millis(100),
        );

        assert!(matches!(result, Err(CommandFailure::TimedOut)));
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn a_program_that_does_not_exist_is_not_started() {
        let result = run_bounded(
            &mut Command::new("/nonexistent/program"),
            Stdout::Capture,
            GENEROUS,
        );
        assert!(matches!(result, Err(CommandFailure::NotStarted(_))));
    }
}
