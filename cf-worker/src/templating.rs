pub use minijinja::context;
use minijinja::{value::Value, Environment};

pub fn apply_template<'a>(asset_name: &str, asset: &str, ctx: Value) -> String {
    let mut env = Environment::new();
    env.add_template(&asset_name, &asset).unwrap();
    let tmpl = env.get_template(&asset_name).unwrap();

    tmpl.render(ctx).unwrap()
}
