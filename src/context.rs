use crate::config::Config;

pub struct Context<'a> {
    pub config: &'a mut (dyn Config + 'a),
    pub io: crate::iostreams::IoStreams,
}

impl Context<'_> {
    pub fn new(config: &mut dyn Config) -> Context {
        // Let's get our IO streams.
        let mut io = crate::iostreams::IoStreams::system();

        // Set the prompt.
        let prompt = config.get("", "prompt").unwrap();
        if prompt == "disabled" {
            io.set_never_prompt(true)
        }

        // Set the pager.
        // Pager precedence
        // 1. OXIDE_PAGER
        // 2. pager from config
        // 3. PAGER
        if let Ok(oxide_pager) = std::env::var("OXIDE_PAGER") {
            io.set_pager(oxide_pager);
        } else if let Ok(pager) = config.get("", "pager") {
            if !pager.is_empty() {
                io.set_pager(pager);
            }
        }

        Context { config, io }
    }
}

#[cfg(test)]
mod test {
    use test_context::{test_context, TestContext};

    use super::*;

    struct TContext {
        orig_oxide_pager_env: Result<String, std::env::VarError>,
    }

    impl TestContext for TContext {
        fn setup() -> TContext {
            TContext {
                orig_oxide_pager_env: std::env::var("OXIDE_PAGER"),
            }
        }

        fn teardown(self) {
            // Put the original env var back.
            if let Ok(ref val) = self.orig_oxide_pager_env {
                std::env::set_var("OXIDE_PAGER", val);
            } else {
                std::env::remove_var("OXIDE_PAGER");
            }
        }
    }

    pub struct TestItem {
        name: String,
        oxide_pager_env: String,
        pager: String,
        prompt: String,
        want_pager: String,
        want_prompt: String,
    }

    #[test_context(TContext)]
    #[test]
    fn test_context() {
        let tests = vec![
            TestItem {
                name: "OXIDE_PAGER env".to_string(),
                oxide_pager_env: "more".to_string(),
                prompt: "".to_string(),
                pager: "".to_string(),
                want_pager: "more".to_string(),
                want_prompt: "enabled".to_string(),
            },
            TestItem {
                name: "OXIDE_PAGER env override".to_string(),
                oxide_pager_env: "more".to_string(),
                prompt: "".to_string(),
                pager: "less".to_string(),
                want_pager: "more".to_string(),
                want_prompt: "enabled".to_string(),
            },
            TestItem {
                name: "config pager".to_string(),
                oxide_pager_env: "".to_string(),
                prompt: "".to_string(),
                pager: "less".to_string(),
                want_pager: "less".to_string(),
                want_prompt: "enabled".to_string(),
            },
            TestItem {
                name: "config prompt".to_string(),
                oxide_pager_env: "".to_string(),
                prompt: "disabled".to_string(),
                pager: "less".to_string(),
                want_pager: "less".to_string(),
                want_prompt: "disabled".to_string(),
            },
        ];

        for t in tests {
            let mut config = crate::config::new_blank_config().unwrap();
            let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);

            if !t.pager.is_empty() {
                c.set("", "pager", &t.pager).unwrap();
            }

            if !t.prompt.is_empty() {
                c.set("", "prompt", &t.prompt).unwrap();
            }

            if !t.oxide_pager_env.is_empty() {
                std::env::set_var("OXIDE_PAGER", t.oxide_pager_env.clone());
            } else {
                std::env::remove_var("OXIDE_PAGER");
            }

            let ctx = Context::new(&mut c);

            assert_eq!(ctx.io.get_pager(), t.want_pager);

            assert_eq!(ctx.io.get_never_prompt(), t.want_prompt == "disabled");

            assert_eq!(ctx.config.get("", "pager").unwrap(), t.want_pager);
            assert_eq!(ctx.config.get("", "prompt").unwrap(), t.want_prompt);
        }
    }
}
