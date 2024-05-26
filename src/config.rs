use std::path::PathBuf;

pub fn find_user_config() -> Option<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        if config_dir.exists() && config_dir.is_dir() {
            let mut pelp_config_dir = config_dir.clone();
            pelp_config_dir.push("pelp");
            // Check if pelp_config_dir
            if pelp_config_dir.exists() && pelp_config_dir.is_dir() {
                let mut pelp_user_config = config_dir.clone();
                pelp_user_config.push("pelp.toml");
                if pelp_user_config.exists() && pelp_user_config.is_file() {
                    return Some(pelp_user_config);
                };
            };
        };
    };
    None
}
