use crate::config_file::get_env_var;

pub fn env_color_disabled() -> bool {
    !get_env_var("NO_COLOR").is_empty() || get_env_var("CLICOLOR") == "0"
}

pub fn env_color_forced() -> bool {
    !get_env_var("CLICOLOR_FORCE").is_empty() && get_env_var("CLICOLOR_FORCE") != "0"
}

pub fn is_true_color_supported() -> bool {
    let term = get_env_var("TERM");
    let color_term = get_env_var("COLORTERM");

    term.contains("24bit")
        || term.contains("truecolor")
        || color_term.contains("24bit")
        || color_term.contains("truecolor")
}

pub fn is_256_color_supported() -> bool {
    let term = get_env_var("TERM");
    let color_term = get_env_var("COLORTERM");

    is_true_color_supported() || term.contains("256") || color_term.contains("256")
}

pub struct ColorScheme {
    enabled: bool,
    is_256_enabled: bool,
    has_true_color: bool,
}

impl ColorScheme {
    pub fn new(enabled: bool, is_256_enabled: bool, has_true_color: bool) -> Self {
        ColorScheme {
            enabled,
            is_256_enabled,
            has_true_color,
        }
    }

    pub fn bold(&self, t: &str) -> String {
        if !self.enabled {
            return t.to_string();
        }

        ansi_term::Style::new().bold().paint(t).to_string()
    }

    pub fn red(&self, t: &str) -> String {
        if !self.enabled {
            return t.to_string();
        }

        ansi_term::Colour::Red.paint(t).to_string()
    }

    pub fn yellow(&self, t: &str) -> String {
        if !self.enabled {
            return t.to_string();
        }

        ansi_term::Colour::Yellow.paint(t).to_string()
    }

    pub fn green(&self, t: &str) -> String {
        if !self.enabled {
            return t.to_string();
        }

        ansi_term::Colour::Green.paint(t).to_string()
    }

    pub fn gray(&self, t: &str) -> String {
        if !self.enabled {
            return t.to_string();
        }

        ansi_term::Colour::Fixed(242).paint(t).to_string()
    }

    pub fn purple(&self, t: &str) -> String {
        if !self.enabled {
            return t.to_string();
        }

        ansi_term::Colour::Purple.paint(t).to_string()
    }

    pub fn blue(&self, t: &str) -> String {
        if !self.enabled {
            return t.to_string();
        }

        ansi_term::Colour::Blue.paint(t).to_string()
    }

    pub fn cyan(&self, t: &str) -> String {
        if !self.enabled {
            return t.to_string();
        }

        ansi_term::Colour::Cyan.paint(t).to_string()
    }

    pub fn success_icon(&self) -> String {
        self.green("✔")
    }

    pub fn success_icon_with_color(&self, color: ansi_term::Colour) -> String {
        color.paint("✔").to_string()
    }

    pub fn warning_icon(&self) -> String {
        self.yellow("!")
    }

    pub fn failure_icon(&self) -> String {
        self.red("✘")
    }
}
