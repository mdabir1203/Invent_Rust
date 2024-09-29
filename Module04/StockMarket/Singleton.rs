use std::collections::HashMap;
use std::sync::{Arc, Mutex, Once};
use std::fs;
use std::io;
use std::thread;

struct ConfigManager {
    config: HashMap<String, String>,
}

impl ConfigManager {
    fn new() -> Result<Self, io::Error> {
        let config_str = fs::read_to_string("config.toml")?;
        let config = parse_config(&config_str);
        Ok(ConfigManager { config })
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    fn set(&mut self, key: String, value: String) {
        self.config.insert(key, value);
    }

    fn save(&self) -> Result<(), io::Error> {
        let config_str = format_config(&self.config);
        fs::write("config.toml", config_str)
    }
}

fn get_config() -> Arc<Mutex<ConfigManager>> {
    static mut SINGLETON: Option<Arc<Mutex<ConfigManager>>> = None;
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            match ConfigManager::new() {
                Ok(config) => {
                    SINGLETON = Some(Arc::new(Mutex::new(config)));
                }
                Err(e) => {
                    eprintln!("Failed to initialize ConfigManager: {}", e);
                    std::process::exit(1);
                }
            }
        });

        SINGLETON.clone().unwrap()
    }
}

fn parse_config(config_str: &str) -> HashMap<String, String> {
    config_str
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn format_config(config: &HashMap<String, String>) -> String {
    config
        .iter()
        .map(|(k, v)| format!("{} = {}", k, v))
        .collect::<Vec<String>>()
        .join("\n")
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::spawn(move || {
                let config = get_config();
                let mut config = config.lock().unwrap();

                let db_url = config.get("database_url").cloned().unwrap_or_default();
                println!("Thread {} read database_url: {}", i, db_url);

                let new_value = format!("new_value_from_thread_{}", i);
                config.set(format!("key_from_thread_{}", i), new_value.clone());
                println!("Thread {} set new value: {}", i, new_value);

                thread::sleep(std::time::Duration::from_millis(100));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let config = get_config();
    let config = config.lock().unwrap();
    config.save()?;
    println!("Configuration saved.");

    Ok(())
}
