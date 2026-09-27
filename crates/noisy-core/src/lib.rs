pub mod compress;
pub mod config;
pub mod frame;
pub mod modulation;
pub mod tx;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper() {
        let config = config::ModemConfig::default();
        println!("Config: {}", config.samples_per_bit(44_800));
        assert_eq!(config.samples_per_bit(44800), 448);
    }
}
