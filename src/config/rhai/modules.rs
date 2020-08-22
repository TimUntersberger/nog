use rhai::module_resolvers::StaticModuleResolver;

mod components;
mod os;

pub fn new() -> StaticModuleResolver {
    let mut resolver = StaticModuleResolver::new();

    resolver.insert("nog/components", components::new());
    resolver.insert("nog/os", os::new());

    resolver
}
