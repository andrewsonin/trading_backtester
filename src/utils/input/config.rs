pub mod from_yaml;
pub mod from_structs;

pub trait FromConfig<Config> {
    fn from_config(config: &Config) -> Self;
}