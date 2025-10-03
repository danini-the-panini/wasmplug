use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use wasmtime::{
    Config, Engine, Store,
    component::{Component, HasSelf, Linker, Resource, bindgen},
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

use crate::danini::wasmplug::logging;

bindgen!({
    world: "foo",
    exports: {
        "danini:wasmplug/plugin/init": async,
        "danini:wasmplug/lorem/generate": async,
    },
    imports: { default: async | trappable },
    with: {
        "danini:wasmplug/logging/logger": FooLogger
    }
});

pub type PluginId = usize;

pub struct FooLogger {
    pub max_level: logging::Level,
}

#[derive(Default)]
pub struct PluginState {
    pub ctx: WasiCtx,
    pub table: ResourceTable,
}

impl WasiView for PluginState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

impl logging::Host for PluginState {}

impl logging::HostLogger for PluginState {
    async fn new(&mut self, max_level: logging::Level) -> wasmtime::Result<Resource<FooLogger>> {
        let id = self.table.push(FooLogger { max_level })?;
        Ok(id)
    }

    async fn get_max_level(
        &mut self,
        logger: Resource<FooLogger>,
    ) -> wasmtime::Result<logging::Level> {
        debug_assert!(!logger.owned());
        let logger = self.table.get(&logger)?;
        Ok(logger.max_level)
    }

    async fn set_max_level(
        &mut self,
        logger: Resource<FooLogger>,
        level: logging::Level,
    ) -> wasmtime::Result<()> {
        debug_assert!(!logger.owned());
        let logger = self.table.get_mut(&logger)?;
        logger.max_level = level;
        Ok(())
    }

    async fn log(
        &mut self,
        logger: Resource<FooLogger>,
        level: logging::Level,
        msg: String,
    ) -> wasmtime::Result<()> {
        debug_assert!(!logger.owned());
        let logger = self.table.get_mut(&logger)?;
        if (level as u32) <= (logger.max_level as u32) {
            println!("{msg}");
        }
        Ok(())
    }

    async fn drop(&mut self, logger: Resource<FooLogger>) -> wasmtime::Result<()> {
        debug_assert!(logger.owned());
        let _logger: FooLogger = self.table.delete(logger)?;
        // ... custom destruction logic here if necessary, otherwise
        // a `Drop for MyLogger` would also work.
        Ok(())
    }
}

pub struct PluginDesc {
    pub name: String,
    pub path: PathBuf,
}

pub async fn load_plugin(
    engine: &Engine,
    linker: &Linker<PluginState>,
    path: PathBuf,
) -> wasmtime::Result<()> {
    let component = Component::from_file(engine, &path)?;

    let mut store = Store::new(
        engine,
        PluginState {
            ctx: Default::default(),
            table: ResourceTable::new(),
        },
    );
    let plugin = Foo::instantiate_async(&mut store, &component, linker).await?;
    let plugin_int = plugin.danini_wasmplug_plugin();
    plugin_int.call_init(&mut store).await?;
    let name = plugin_int.call_get_name(&mut store)?;

    println!("Running plugin {}...", name);

    let lorem_int = plugin.danini_wasmplug_lorem();

    let lorem = lorem_int
        .call_generate(&mut store, Some(3), None, None)
        .await?;

    println!("=====");
    println!("{}", lorem.unwrap_or_else(|e| e.to_string()));
    println!("=====");

    Ok(())
}

pub async fn load_plugins(plugins_dir: &Path) -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.async_support(true);
    let engine = Engine::new(&config)?;
    let mut linker: Linker<PluginState> = Linker::new(&engine);

    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    Foo::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

    if !plugins_dir.is_dir() {
        return Err(wasmtime::Error::msg("Plugins directory does not exist"));
    }

    for entry in fs::read_dir(plugins_dir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("wasm") {
            load_plugin(&engine, &linker, path).await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> wasmtime::Result<()> {
    println!("Hello, world!");

    load_plugins(&Path::new("plugins")).await?;

    Ok(())
}
