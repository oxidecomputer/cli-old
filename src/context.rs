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
