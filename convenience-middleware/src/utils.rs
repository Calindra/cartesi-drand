pub mod util {
    use std::{error::Error, path::Path};

    use serde_json::Value;
    use tokio::fs::read_to_string;

    use crate::models::models::DrandEnv;
    pub(crate) fn deserialize_obj(request: &str) -> Option<serde_json::Map<String, Value>> {
        let json = serde_json::from_str::<serde_json::Value>(request);

        match json {
            Ok(Value::Object(map)) => Some(map),
            _ => None,
        }
    }

    pub async fn write_env_to_json(drand: &DrandEnv) -> Result<(), Box<dyn Error>> {
        let path = Path::new("drand.config.json");
        let content = serde_json::to_string_pretty(drand)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    pub async fn load_env_from_memory(drand: DrandEnv) {
        std::env::set_var("DRAND_PUBLIC_KEY", drand.DRAND_PUBLIC_KEY);

        if let Some(period) = drand.DRAND_PERIOD {
            std::env::set_var("DRAND_PERIOD", period.to_string());
        }

        if let Some(genesis_time) = drand.DRAND_GENESIS_TIME {
            std::env::set_var("DRAND_GENESIS_TIME", genesis_time.to_string());
        }

        if let Some(safe_seconds) = drand.DRAND_SAFE_SECONDS {
            std::env::set_var("DRAND_SAFE_SECONDS", safe_seconds.to_string());
        }
    }

    pub async fn load_env_from_json() -> Result<(), Box<dyn Error>> {
        println!("Loading env from json");

        let path = Path::new("drand.config.json");
        if !path.exists() {
            return Err("File not found".into());
        }
        let content = read_to_string(path).await?;
        let json = serde_json::from_str::<DrandEnv>(&content)?;

        load_env_from_memory(json).await;

        Ok(())
    }
}
