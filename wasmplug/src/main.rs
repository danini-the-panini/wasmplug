use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use wasmtime::{
    Engine, Store,
    component::{Component, HasSelf, Linker, bindgen},
};

use crate::danini::wasmplug::host::{Host, Thing};

bindgen!("foo");

pub type PluginId = usize;

pub struct FooState {
    pub next_plugin_id: PluginId,
    pub plugin_descs: HashMap<PluginId, PluginDesc>,
}

impl FooState {
    pub fn new() -> Self {
        Self {
            next_plugin_id: 1,
            plugin_descs: Default::default(),
        }
    }

    pub fn next_id(&mut self) -> PluginId {
        let id = self.next_plugin_id;
        self.next_plugin_id += 1;
        id
    }
}

pub struct PluginState {
    pub plugin_id: PluginId,
}

impl Host for PluginState {
    fn do_the_thing(&mut self, t: Thing) {
        println!("Doing the thing: {}!", t.name);
    }
}

pub struct PluginDesc {
    pub name: String,
    pub path: PathBuf,
}

pub fn load_plugin(
    state: &mut FooState,
    engine: &Engine,
    linker: &Linker<PluginState>,
    path: PathBuf,
) -> wasmtime::Result<()> {
    let component = Component::from_file(engine, &path)?;

    let plugin_id = state.next_id();

    let plugin_name = {
        let mut store = Store::new(engine, PluginState { plugin_id });
        let plugin = Foo::instantiate(&mut store, &component, linker)?;
        plugin.call_init(&mut store)?;
        plugin.call_get_name(&mut store)?
    };

    state.plugin_descs.insert(
        plugin_id,
        PluginDesc {
            name: plugin_name,
            path,
        },
    );

    Ok(())
}

pub fn load_plugins(state: &mut FooState, plugins_dir: &Path) -> wasmtime::Result<()> {
    let engine = Engine::default();
    let mut linker: Linker<PluginState> = Linker::new(&engine);

    Foo::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

    if !plugins_dir.is_dir() {
        return Err(wasmtime::Error::msg("Plugins directory does not exist"));
    }

    for entry in fs::read_dir(plugins_dir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("wasm") {
            load_plugin(state, &engine, &linker, path)?;
        }
    }

    Ok(())
}

fn main() {
    println!("Hello, world!");
    let mut state = FooState::new();
    load_plugins(&mut state, &Path::new("plugins")).expect("Failed to load plugins!");
}
