use anyhow::{anyhow, Result};
use terminal_size::{terminal_size, Height, Width};

const DEFAULT_WIDTH: i32 = 80;

pub struct IoStreams {
    pub stdin: Box<dyn std::io::Read + Send + Sync>,
    pub out: Box<dyn std::io::Write + Send + Sync>,
    pub err_out: Box<dyn std::io::Write + Send + Sync>,

    color_enabled: bool,
    is_256_enabled: bool,
    has_true_color: bool,
    terminal_theme: String,

    progress_indicator_enabled: bool,

    stdin_tty_override: bool,
    stdin_is_tty: bool,
    stdout_tty_override: bool,
    stdout_is_tty: bool,
    stderr_tty_override: bool,
    stderr_is_tty: bool,

    terminal_width_override: i32,
    tty_size: fn() -> Result<(i32, i32)>,

    never_prompt: bool,

    pub tmp_file_override: Option<std::fs::File>,
}

impl IoStreams {
    pub fn color_enabled(&self) -> bool {
        self.color_enabled
    }

    pub fn color_support_256(&self) -> bool {
        self.is_256_enabled
    }

    pub fn has_true_color(&self) -> bool {
        self.has_true_color
    }

    #[allow(dead_code)]
    pub fn detect_terminal_theme(&mut self) -> String {
        if !self.color_enabled() {
            self.terminal_theme = "none".to_string();
            return self.terminal_theme.to_string();
        }

        let timeout = std::time::Duration::from_millis(100);
        match termbg::theme(timeout) {
            Ok(theme) => {
                if matches!(theme, termbg::Theme::Dark) {
                    self.terminal_theme = "dark".to_string();
                    return self.terminal_theme.to_string();
                }

                self.terminal_theme = "light".to_string();
                self.terminal_theme.to_string()
            }
            Err(_) => {
                self.terminal_theme = "none".to_string();
                self.terminal_theme.to_string()
            }
        }
    }

    #[allow(dead_code)]
    pub fn terminal_theme(&self) -> String {
        if self.terminal_theme.is_empty() {
            return "none".to_string();
        }
        self.terminal_theme.to_string()
    }

    #[allow(dead_code)]
    pub fn set_color_enabled(&mut self, color_enabled: bool) {
        self.color_enabled = color_enabled;
    }

    #[allow(dead_code)]
    pub fn set_stdin_tty(&mut self, is_tty: bool) {
        self.stdin_tty_override = true;
        self.stdin_is_tty = is_tty;
    }

    #[cfg(target_os = "windows")]
    // TODO: actually implement a real check for windows.
    pub fn is_stdin_tty(&self) -> bool {
        if self.stdin_tty_override {
            return self.stdin_is_tty;
        }

        true
    }

    #[cfg(not(target_os = "windows"))]
    pub fn is_stdin_tty(&self) -> bool {
        if self.stdin_tty_override {
            return self.stdin_is_tty;
        }

        atty::is(atty::Stream::Stdin)
    }

    pub fn set_stdout_tty(&mut self, is_tty: bool) {
        self.stdout_tty_override = true;
        self.stdout_is_tty = is_tty;
    }

    pub fn is_stdout_tty(&self) -> bool {
        if self.stdout_tty_override {
            return self.stdout_is_tty;
        }

        atty::is(atty::Stream::Stdout)
    }

    pub fn set_stderr_tty(&mut self, is_tty: bool) {
        self.stderr_tty_override = true;
        self.stderr_is_tty = is_tty;
    }

    #[allow(dead_code)]
    pub fn is_stderr_tty(&self) -> bool {
        if self.stderr_tty_override {
            return self.stderr_is_tty;
        }

        atty::is(atty::Stream::Stderr)
    }

    pub fn can_prompt(&self) -> bool {
        if self.never_prompt {
            return false;
        }

        self.is_stdin_tty() && self.is_stdout_tty()
    }

    #[cfg(test)]
    pub fn get_never_prompt(&self) -> bool {
        self.never_prompt
    }

    pub fn set_never_prompt(&mut self, never_prompt: bool) {
        self.never_prompt = never_prompt;
    }

    #[allow(dead_code)]
    /// This returns a handle to a spinner. To stop the spinner, call `.stop()` on it.
    pub fn start_process_indicator(&mut self) -> Option<terminal_spinners::SpinnerHandle> {
        self.start_process_indicator_with_label("")
    }

    /// This returns a handle to a spinner. To stop the spinner, call `.stop()` on it.
    pub fn start_process_indicator_with_label(&mut self, label: &str) -> Option<terminal_spinners::SpinnerHandle> {
        if !self.progress_indicator_enabled {
            return None;
        }

        let pi = terminal_spinners::SpinnerBuilder::new()
            .spinner(&terminal_spinners::DOTS11)
            .text(label.to_string());

        Some(pi.start())
    }

    #[allow(dead_code)]
    pub fn terminal_width(&self) -> i32 {
        if self.terminal_width_override > 0 {
            return self.terminal_width_override;
        }

        let (w, _) = tty_size().unwrap_or((DEFAULT_WIDTH, 0));
        w
    }

    pub fn force_terminal(&mut self, spec: &str) {
        self.color_enabled = !crate::colors::env_color_disabled();
        self.set_stdout_tty(true);

        if let Ok(i) = spec.parse::<i32>() {
            self.terminal_width_override = i;
            return;
        }

        let ts = (self.tty_size)();
        if let Ok((w, _)) = ts {
            self.terminal_width_override = w;
        } else {
            return;
        }

        if spec.ends_with('%') {
            if let Ok(p) = spec.trim_end_matches('%').parse::<f64>() {
                self.terminal_width_override = ((self.terminal_width_override as f64) * (p / 100.00)) as i32;
            }
        }
    }

    pub fn color_scheme(&self) -> crate::colors::ColorScheme {
        crate::colors::ColorScheme::new(self.color_enabled(), self.color_support_256(), self.has_true_color())
    }

    pub fn write_output_for_vec<T: serde::Serialize + tabled::Tabled>(
        &mut self,
        format: &crate::types::FormatOutput,
        value: impl IntoIterator<Item = T> + serde::Serialize,
    ) -> Result<()> {
        match format {
            crate::types::FormatOutput::Json => self.write_output_json(&serde_json::to_value(value)?),
            crate::types::FormatOutput::Table => self.write_output_table_for_vec(value),
            crate::types::FormatOutput::Yaml => self.write_output_yaml(&value),
        }
    }

    pub fn write_output<T: serde::Serialize + tabled::Tabled>(
        &mut self,
        format: &crate::types::FormatOutput,
        value: &T,
    ) -> Result<()> {
        match format {
            crate::types::FormatOutput::Json => self.write_output_json(&serde_json::to_value(value)?),
            crate::types::FormatOutput::Table => self.write_output_table(value),
            crate::types::FormatOutput::Yaml => self.write_output_yaml(value),
        }
    }

    pub fn write_output_json(&mut self, json: &serde_json::Value) -> Result<()> {
        if self.color_enabled() {
            // Print the response body.
            writeln!(self.out, "{}", colored_json::to_colored_json_auto(json)?)?;
        } else {
            // Print the response body.
            writeln!(self.out, "{}", serde_json::to_string_pretty(json)?)?;
        }

        Ok(())
    }

    pub fn write_output_yaml<Y: serde::Serialize>(&mut self, yaml: &Y) -> Result<()> {
        // Print the response body.
        writeln!(self.out, "{}", serde_yaml::to_string(yaml)?)?;

        Ok(())
    }

    pub fn write_output_table_for_vec<T: tabled::Tabled>(&mut self, value: impl IntoIterator<Item = T>) -> Result<()> {
        let table = tabled::Table::new(value).with(tabled::Style::psql()).to_string();

        writeln!(self.out, "{}", table)?;

        Ok(())
    }

    pub fn write_output_table<T: tabled::Tabled>(&mut self, value: &T) -> Result<()> {
        let table = tabled::Table::new(vec![value])
            .with(tabled::Rotate::Left)
            .with(
                tabled::Modify::new(tabled::Full)
                    .with(tabled::Alignment::left())
                    .with(tabled::Alignment::top()),
            )
            .with(tabled::Style::psql().header_off())
            .to_string();

        writeln!(self.out, "{}", table)?;

        Ok(())
    }

    pub fn system() -> Self {
        let stdout_is_tty = atty::is(atty::Stream::Stdout);
        let stderr_is_tty = atty::is(atty::Stream::Stderr);

        #[cfg(windows)]
        let mut assume_true_color = false;
        #[cfg(unix)]
        let assume_true_color = false;
        if stdout_is_tty {
            // Note for Windows 10 users: On Windows 10, the application must enable ANSI support
            // first.
            #[cfg(windows)]
            let enabled = ansi_term::enable_ansi_support();
            #[cfg(windows)]
            if enabled.is_ok() {
                assume_true_color = true;
            }

            // Enable colored json output.
            #[cfg(windows)]
            let enabled = colored_json::enable_ansi_support();
        }

        let mut io = IoStreams {
            stdin: Box::new(std::io::stdin()),
            out: Box::new(std::io::stdout()),
            err_out: Box::new(std::io::stderr()),
            color_enabled: crate::colors::env_color_forced() || (!crate::colors::env_color_disabled() && stdout_is_tty),
            is_256_enabled: assume_true_color || crate::colors::is_256_color_supported(),
            has_true_color: assume_true_color || crate::colors::is_true_color_supported(),

            terminal_theme: "".to_string(),

            progress_indicator_enabled: false,

            stdin_tty_override: false,
            stdin_is_tty: atty::is(atty::Stream::Stdin),
            stdout_tty_override: false,
            stdout_is_tty,
            stderr_tty_override: false,
            stderr_is_tty,

            terminal_width_override: 0,

            tty_size,

            never_prompt: false,
            tmp_file_override: None,
        };

        if stdout_is_tty && stderr_is_tty {
            io.progress_indicator_enabled = true;
        }

        // prevent duplicate is_terminal queries now that we know the answer.
        io.set_stdout_tty(stdout_is_tty);
        io.set_stderr_tty(stderr_is_tty);

        io
    }

    #[cfg(test)]
    pub fn test() -> (Self, String, String) {
        let mut io = IoStreams::system();

        let (stdout, stdout_path) = tempfile::NamedTempFile::new().unwrap().keep().unwrap();
        let (stderr, stderr_path) = tempfile::NamedTempFile::new().unwrap().keep().unwrap();

        io.out = Box::new(stdout);
        io.err_out = Box::new(stderr);

        io.tty_size = test_tty_size;

        (
            io,
            stdout_path.into_os_string().into_string().unwrap(),
            stderr_path.into_os_string().into_string().unwrap(),
        )
    }
}

#[cfg(test)]
fn test_tty_size() -> Result<(i32, i32)> {
    Err(anyhow::anyhow!("tty_size not implemented in tests"))
}

// tty_size measures the size of the controlling terminal for the current process.
fn tty_size() -> Result<(i32, i32)> {
    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        Ok((w.into(), h.into()))
    } else {
        Err(anyhow!("Failed to get terminal size"))
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    pub struct TestItem {
        name: String,
        io: IoStreams,
        arg: String,
        want_tty: bool,
        want_width: i32,
    }

    fn measure_width_fn() -> Result<(i32, i32)> {
        Ok((72, 0))
    }

    fn measure_width_fails_fn() -> Result<(i32, i32)> {
        Err(anyhow!("Failed to get terminal size"))
    }

    #[test]
    fn test_force_terminal() {
        let mut measure_width = IoStreams::system();
        measure_width.tty_size = measure_width_fn;

        let mut measure_width_fails = IoStreams::system();
        measure_width_fails.tty_size = measure_width_fails_fn;

        let mut apply_percentage = IoStreams::system();
        apply_percentage.tty_size = measure_width_fn;

        let tests = vec![
            TestItem {
                name: "explicit width".to_string(),
                io: IoStreams::system(),
                arg: "72".to_string(),
                want_tty: true,
                want_width: 72,
            },
            TestItem {
                name: "measure width".to_string(),
                io: measure_width,
                arg: "true".to_string(),
                want_tty: true,
                want_width: 72,
            },
            /*TestItem {
                name: "measure width fails".to_string(),
                io: measure_width_fails,
                arg: "true".to_string(),
                want_tty: true,
                want_width: 80,
            },*/
            TestItem {
                name: "apply percentage".to_string(),
                io: apply_percentage,
                arg: "50%".to_string(),
                want_tty: true,
                want_width: 36,
            },
        ];

        for mut t in tests {
            t.io.force_terminal(&t.arg);
            let is_tty = t.io.is_stdout_tty();
            assert_eq!(is_tty, t.want_tty, "test {}", t.name);

            let width = t.io.terminal_width();
            assert_eq!(width, t.want_width, "test {}", t.name);
        }
    }
}
