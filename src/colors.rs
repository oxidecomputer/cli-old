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

#[cfg(test)]
mod test {
    use test_context::{test_context, TestContext};

    use super::*;

    struct Context {
        orig_NO_COLOR: Result<String, std::env::VarError>,
        orig_CLICOLOR: Result<String, std::env::VarError>,
        orig_CLICOLOR_FORCE: Result<String, std::env::VarError>,
    }

    impl TestContext for Context {
        fn setup() -> Context {
            Context {
                orig_NO_COLOR: std::env::var("NO_COLOR"),
                orig_CLICOLOR: std::env::var("CLICOLOR"),
                orig_CLICOLOR_FORCE: std::env::var("CLICOLOR_FORCE"),
            }
        }

        fn teardown(self) {
            // Put the original env var back.
            if let Ok(ref val) = self.orig_NO_COLOR {
                std::env::set_var("NO_COLOR", val);
            } else {
                std::env::remove_var("NO_COLOR");
            }

            if let Ok(ref val) = self.orig_CLICOLOR {
                std::env::set_var("CLICOLOR", val);
            } else {
                std::env::remove_var("CLICOLOR");
            }

            if let Ok(ref val) = self.orig_CLICOLOR_FORCE {
                std::env::set_var("CLICOLOR_FORCE", val);
            } else {
                std::env::remove_var("CLICOLOR_FORCE");
            }
        }
    }

    pub struct TestItem {
        name: String,
        NO_COLOR: String,
        CLICOLOR: String,
        CLICOLOR_FORCE: String,
        want: bool,
    }

    #[test_context(Context)]
    #[test]
    fn test_env_color_disabled() {
        let tests = vec![
            TestItem {
                name: "pristine env".to_string(),
                NO_COLOR: "".to_string(),
                CLICOLOR: "".to_string(),
                CLICOLOR_FORCE: "".to_string(),
                want: false,
            },
            TestItem {
                name: "NO_COLOR enabled".to_string(),
                NO_COLOR: "1".to_string(),
                CLICOLOR: "".to_string(),
                CLICOLOR_FORCE: "".to_string(),
                want: true,
            },
            TestItem {
                name: "CLICOLOR disabled".to_string(),
                NO_COLOR: "".to_string(),
                CLICOLOR: "0".to_string(),
                CLICOLOR_FORCE: "".to_string(),
                want: true,
            },
            TestItem {
                name: "CLICOLOR enabled".to_string(),
                NO_COLOR: "".to_string(),
                CLICOLOR: "1".to_string(),
                CLICOLOR_FORCE: "".to_string(),
                want: false,
            },
            TestItem {
                name: "CLICOLOR_FORCE has no effect".to_string(),
                NO_COLOR: "".to_string(),
                CLICOLOR: "".to_string(),
                CLICOLOR_FORCE: "1".to_string(),
                want: false,
            },
        ];

        for t in tests {
            std::env::set_var("NO_COLOR", t.NO_COLOR);
            std::env::set_var("CLICOLOR", t.CLICOLOR);
            std::env::set_var("CLICOLOR_FORCE", t.CLICOLOR_FORCE);

            let got = env_color_disabled();
            assert_eq!(got, t.want, "test {}", t.name);
        }
    }

    #[test_context(Context)]
    #[test]
    fn test_env_color_forced() {
        let tests = vec![
            TestItem {
                name: "pristine env".to_string(),
                NO_COLOR: "".to_string(),
                CLICOLOR: "".to_string(),
                CLICOLOR_FORCE: "".to_string(),
                want: false,
            },
            TestItem {
                name: "NO_COLOR enabled".to_string(),
                NO_COLOR: "1".to_string(),
                CLICOLOR: "".to_string(),
                CLICOLOR_FORCE: "".to_string(),
                want: false,
            },
            TestItem {
                name: "CLICOLOR disabled".to_string(),
                NO_COLOR: "".to_string(),
                CLICOLOR: "0".to_string(),
                CLICOLOR_FORCE: "".to_string(),
                want: false,
            },
            TestItem {
                name: "CLICOLOR enabled".to_string(),
                NO_COLOR: "".to_string(),
                CLICOLOR: "1".to_string(),
                CLICOLOR_FORCE: "".to_string(),
                want: false,
            },
            TestItem {
                name: "CLICOLOR_FORCE enabled".to_string(),
                NO_COLOR: "".to_string(),
                CLICOLOR: "".to_string(),
                CLICOLOR_FORCE: "1".to_string(),
                want: true,
            },
            TestItem {
                name: "CLICOLOR_FORCE disabled".to_string(),
                NO_COLOR: "".to_string(),
                CLICOLOR: "".to_string(),
                CLICOLOR_FORCE: "0".to_string(),
                want: false,
            },
        ];

        for t in tests {
            std::env::set_var("NO_COLOR", t.NO_COLOR);
            std::env::set_var("CLICOLOR", t.CLICOLOR);
            std::env::set_var("CLICOLOR_FORCE", t.CLICOLOR_FORCE);

            let got = env_color_forced();

            assert_eq!(got, t.want, "test {}", t.name);
        }
    }
}
