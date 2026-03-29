use fides_macro::plugin_interface;

#[plugin_interface(version = 1, buffer = CallerAllocated)]
pub trait BadPlugin: Send + Sync {
    fn do_thing(&self) -> String;
}

fn main() {}
