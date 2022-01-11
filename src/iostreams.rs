use std::{
    collections::HashMap,
    env,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Result};
use terminal_size::{terminal_size, Height, Width};

use crate::config_file::get_env_var;

const DEFAULT_WIDTH: i32 = 80;

pub struct IoStreams<'a> {
    pub stdin: &'a (dyn std::io::Read + 'a),
    pub out: std::io::BufWriter<&'a mut dyn std::io::Write>,
    pub err_out: &'a (dyn std::io::Write + 'a),

    // the original (non-colorable) output stream
    original_out: &'a (dyn std::io::Write + 'a),

    color_enabled: bool,
    is_256_enabled: bool,
    has_true_color: bool,
    terminal_theme: String,

    progress_indicator_enabled: bool,
    progress_indicator: Option<terminal_spinners::SpinnerHandle>,

    stdin_tty_override: bool,
    stdin_is_tty: bool,
    stdout_tty_override: bool,
    stdout_is_tty: bool,
    stderr_tty_override: bool,
    stderr_is_tty: bool,

    terminal_width_override: i32,
    tty_size: fn() -> Result<(i32, i32)>,

    pager_command: String,
    pager_process: Option<std::process::Child>,

    never_prompt: bool,

    pub tmp_file_override: Option<std::fs::File>,
}

impl IoStreams<'_> {
    pub fn color_enabled(&self) -> bool {
        self.color_enabled
    }

    pub fn color_support_256(&self) -> bool {
        self.is_256_enabled
    }

    pub fn has_true_color(&self) -> bool {
        self.has_true_color
    }

    pub fn detect_terminal_theme(&mut self) -> String {
        if !self.color_enabled() {
            self.terminal_theme = "none".to_string();
            return self.terminal_theme.to_string();
        }

        if self.pager_process.is_some() {
            self.terminal_theme = "none".to_string();
            return self.terminal_theme.to_string();
        }

        let style = get_env_var("GLAMOUR_STYLE");
        if !style.is_empty() && style != "auto" {
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

    pub fn terminal_theme(&self) -> String {
        if self.terminal_theme.is_empty() {
            return "none".to_string();
        }
        self.terminal_theme.to_string()
    }

    pub fn set_color_enabled(&mut self, color_enabled: bool) {
        self.color_enabled = color_enabled;
    }

    pub fn set_stdin_tty(&mut self, is_tty: bool) {
        self.stdin_tty_override = true;
        self.stdin_is_tty = is_tty;
    }

    // TODO: fix and do others.
    pub fn is_stdin_tty(&self) -> bool {
        if self.stdin_tty_override {
            return self.stdin_is_tty;
        }

        false
    }

    // TODO: fix and do others.
    pub fn is_stdout_tty(&self) -> bool {
        if self.stdout_tty_override {
            return self.stdout_is_tty;
        }

        false
    }

    pub fn set_pager(&mut self, pager_command: String) {
        self.pager_command = pager_command;
    }

    pub fn get_pager(&self) -> String {
        self.pager_command.to_string()
    }

    pub fn start_pager(&mut self) -> Result<()> {
        if self.pager_command.is_empty() || self.pager_command == "cat" || !self.is_stdout_tty() {
            return Ok(());
        }

        let pager_args = shlex::split(&self.pager_command).unwrap_or_default();
        if pager_args.is_empty() {
            return Err(anyhow!("pager command is empty"));
        }

        // Remove PAGER from env.
        let mut filtered_env: HashMap<String, String> = env::vars().filter(|&(ref k, _)| k != "PAGER").collect();

        if !filtered_env.contains_key("LESS") {
            filtered_env.insert("LESS".to_string(), "FRX".to_string());
        }

        if !filtered_env.contains_key("LV") {
            filtered_env.insert("LV".to_string(), "-c".to_string());
        }

        // TODO: fix this.
        let pager_cmd = Command::new(pager_args.first().unwrap())
            .args(pager_args.iter().skip(1))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear()
            .envs(&filtered_env)
            .spawn()
            .expect("failed to execute pager child");

        self.pager_process = Some(pager_cmd);

        Ok(())
    }

    pub fn stop_pager(&mut self) -> Result<()> {
        if self.pager_process.is_none() {
            return Ok(());
        }

        let mut pager_process = self.pager_process.take().unwrap();
        let _ = pager_process.kill();

        Ok(())
    }

    pub fn can_prompt(&self) -> bool {
        if self.never_prompt {
            return false;
        }

        self.is_stdin_tty() && self.is_stdout_tty()
    }

    pub fn get_never_prompt(&self) -> bool {
        self.never_prompt
    }

    pub fn set_never_prompt(&mut self, never_prompt: bool) {
        self.never_prompt = never_prompt;
    }

    pub fn start_process_indicator(&mut self) {
        self.start_process_indicator_with_label("")
    }

    // TODO: do we need a mutex here?
    pub fn start_process_indicator_with_label(&mut self, label: &str) {
        if !self.progress_indicator_enabled {
            return;
        }

        /*if let Some(ref mut progress_indicator) = self.progress_indicator {
            if !label.is_empty() {
                progress_indicator.text(label);
            } else {
                progress_indicator.text("");
            }

            return;
        }*/

        let mut pi = terminal_spinners::SpinnerBuilder::new().spinner(&terminal_spinners::DOTS11);
        if !label.is_empty() {
            pi = pi.prefix(format!("{} ", label));
        }

        self.progress_indicator = Some(pi.start());
    }

    // TODO: do we need a mutex here?
    pub fn stop_progress_indicator(&mut self) {
        if self.progress_indicator.is_none() {
            return;
        }

        let pi = self.progress_indicator.as_ref().unwrap();
        // TODO: fix this.
        //pi.done();
        self.progress_indicator = None;
    }

    // TODO: fix this.
    pub fn terminal_width(&self) -> i32 {
        if self.terminal_width_override > 0 {
            return self.terminal_width_override;
        }

        let (w, _) = tty_size().unwrap_or((DEFAULT_WIDTH, 0));
        w
    }

    pub fn process_terminal_width(&mut self) -> i32 {
        let (w, _) = tty_size().unwrap_or((DEFAULT_WIDTH, 0));

        if w == 0 {
            return DEFAULT_WIDTH;
        }

        w
    }

    pub fn force_terminal(&mut self, spec: &str) {
        self.color_enabled = !crate::colors::env_color_disabled();
        // TODO: fix this.
        // self.set_stdout_tty(true);

        if let Ok(i) = spec.parse::<i32>() {
            self.terminal_width_override = i;
        }

        let ts = tty_size();
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
}

// tty_sdize measures the size of the controlling terminal for the current process.
fn tty_size() -> Result<(i32, i32)> {
    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        Ok((w.into(), h.into()))
    } else {
        Err(anyhow!("Failed to get terminal size"))
    }
}