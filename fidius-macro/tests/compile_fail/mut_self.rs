use fidius_macro::plugin_interface;

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait BadPlugin: Send + Sync {
    fn mutate(&mut self);
}

fn main() {}
