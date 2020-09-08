use rhai::module_resolvers::StaticModuleResolver;

mod components;
mod http;

pub fn new() -> StaticModuleResolver {
    let mut resolver = StaticModuleResolver::new();

    resolver.insert("nog/components", components::new());
    resolver.insert("nog/http", http::new());

    resolver
}
