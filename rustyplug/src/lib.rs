use crate::danini::wasmplug::host;

wit_bindgen::generate!({
    path: "../wasmplug/wit",
    world: "foo"
});

struct Component;

export!(Component);

impl Guest for Component {
    fn init() -> () {
        println!("Init Rustyplug! 🦀");

        let thing = host::Thing {
            name: "Rustything 9000".into(),
            foo: 9000,
        };

        host::do_the_thing(&thing);
    }

    fn get_name() -> _rt::String {
        "Rustyplug".into()
    }
}
