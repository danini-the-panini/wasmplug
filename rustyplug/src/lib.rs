use crate::{
    danini::wasmplug::logging,
    exports::danini::wasmplug::{lorem, plugin},
};

wit_bindgen::generate!({
    path: "../wasmplug/wit",
    world: "foo",
    features: ["async"],
    async: [
        "export:danini:wasmplug/plugin#init",
        "export:danini:wasmplug/lorem#generate"
    ]
});

struct Component;

export!(Component);

impl plugin::Guest for Component {
    async fn init() {
        let log = logging::Logger::new(logging::Level::Warn);
        log.log(logging::Level::Debug, "initializing...");
        println!("Init Rustyplug! 🦀");
    }

    fn get_name() -> _rt::String {
        "Rustyplug".into()
    }
}

impl lorem::Guest for Component {
    async fn generate(
        paras: Option<u32>,
        sentences: Option<u32>,
        options: Option<_rt::Vec<_rt::String>>,
    ) -> Result<_rt::String, _rt::String> {
        let log = logging::Logger::new(logging::Level::Warn);
        log.log(logging::Level::Debug, "generating...");
        Ok(format!(
            "TODO: paras: {:?}, sentences: {:?}, options: {:?}",
            paras, sentences, options
        ))
    }
}
