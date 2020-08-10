use rhai::module_resolvers::StaticModuleResolver;

mod components;

pub fn new() -> StaticModuleResolver {
    let mut resolver = StaticModuleResolver::new();

    resolver.insert("nog/components", components::new());

    resolver
}