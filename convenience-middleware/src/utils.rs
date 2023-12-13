pub mod util {
    use std::{error::Error, path::Path};

    use log::info;
    use serde_json::Value;
    use tokio::fs::read_to_string;

    use crate::models::structs::DrandEnv;

    pub fn generate_payload_hex<T>(json: T) -> Result<String, Box<dyn Error>>
    where
        T: serde::Serialize,
    {
        let data = serde_json::to_string(&json)?;
        // encode lower case hexa
        let encode = format!("0x{}", hex::encode(data));
        Ok(encode)
    }

    pub fn deserialize_obj(request: &str) -> Option<serde_json::Map<String, Value>> {
        let json = serde_json::from_str::<serde_json::Value>(request);

        match json {
            Ok(Value::Object(map)) => Some(map),
            _ => None,
        }
    }

    fn var_string_to_u64(str: &str) -> u64 {
        let err_msg = format!("Var {} is not defined", str);
        let value = std::env::var(str).expect(&err_msg);
        let err_msg = format!("Var {} cannot parse", str);
        value.parse::<u64>().expect(&err_msg)
    }

    pub async fn write_env_to_json() -> Result<(), Box<dyn Error>> {
        let path = Path::new("drand.config.json");

        let drand_env = DrandEnv {
            DRAND_PUBLIC_KEY: std::env::var("DRAND_PUBLIC_KEY").unwrap(),
            DRAND_PERIOD: Some(var_string_to_u64("DRAND_PERIOD")),
            DRAND_GENESIS_TIME: Some(var_string_to_u64("DRAND_GENESIS_TIME")),
            DRAND_SAFE_SECONDS: Some(var_string_to_u64("DRAND_SAFE_SECONDS")),
        };

        let content = serde_json::to_string_pretty(&drand_env)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    pub async fn load_env_from_memory(drand: DrandEnv) {
        info!("DRAND_PUBLIC_KEY={}", &drand.DRAND_PUBLIC_KEY);
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
        info!("Loading env from json");

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
