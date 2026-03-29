use fidius_macro::plugin_interface;

#[plugin_interface(buffer = PluginAllocated)]
pub trait BadPlugin: Send + Sync {
    fn do_thing(&self) -> String;
}

fn main() {}
