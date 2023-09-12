pub mod repository {
    use std::{collections::BTreeMap, error::Error, path::PathBuf};

    use async_trait::async_trait;

    use crate::util::json::{read_json, write_json};

    #[async_trait]
    pub trait RepositoryCrud<T, I>
    where
        T: Clone,
    {
        async fn create(&mut self, id: I, data: T) -> Result<I, Box<dyn Error>>;
        async fn get(&self, id: I) -> Result<T, Box<dyn Error>>;
        async fn update(&mut self, id: I, partial_data: T) -> Result<(), Box<dyn Error>>;
        async fn delete(&mut self, id: I) -> Result<(), Box<dyn Error>>;
    }

    type JSON = serde_json::Map<String, serde_json::Value>;

    pub struct RepositoryStrategyMemory {
        data: BTreeMap<String, JSON>,
    }

    impl Default for RepositoryStrategyMemory {
        fn default() -> Self {
            Self {
                data: BTreeMap::new(),
            }
        }
    }

    impl RepositoryStrategyMemory {
        fn new() -> Self {
            RepositoryStrategyMemory::default()
        }
    }

    #[async_trait]
    impl RepositoryCrud<JSON, String> for RepositoryStrategyMemory {
        async fn create(&mut self, id: String, data: JSON) -> Result<String, Box<dyn Error>> {
            self.data.insert(id.clone(), data);
            Ok(id)
        }

        async fn get(&self, id: String) -> Result<JSON, Box<dyn Error>> {
            let json = self.data.get(&id).ok_or("Not found")?;
            Ok(json.clone())
        }

        async fn update(&mut self, id: String, partial_data: JSON) -> Result<(), Box<dyn Error>> {
            let json_before = self.get(id.clone()).await?;
            let mut json_after = json_before.clone();

            for (key, value) in partial_data {
                json_after.insert(key.to_string(), value.clone());
            }

            self.data.insert(id, json_after);
            Ok(())
        }

        async fn delete(&mut self, id: String) -> Result<(), Box<dyn Error>> {
            self.data.remove(&id);
            Ok(())
        }
    }

    pub struct RepositoryStrategyJSONFile {
        directory: PathBuf,
    }

    impl Default for RepositoryStrategyJSONFile {
        fn default() -> Self {
            let data_folder = String::from("data");
            RepositoryStrategyJSONFile::new(data_folder)
        }
    }

    impl RepositoryStrategyJSONFile {
        pub fn new(directory: String) -> Self {
            Self {
                directory: PathBuf::from(directory),
            }
        }

        pub fn get_directory(&self) -> PathBuf {
            self.directory.clone()
        }

        pub fn get_path(&self, filename: &str) -> String {
            let mut path = self.get_directory();
            path.set_file_name(filename);
            path.set_extension("json");
            path.to_str().unwrap().to_string()
        }
    }

    #[async_trait]
    impl RepositoryCrud<serde_json::Value, String> for RepositoryStrategyJSONFile {
        async fn create(
            &mut self,
            id: String,
            data: serde_json::Value,
        ) -> Result<String, Box<dyn Error>> {
            let path = self.get_path(&id);
            write_json(&path, &data).await?;
            Ok(path)
        }

        async fn get(&self, id: String) -> Result<serde_json::Value, Box<dyn Error>> {
            let path = self.get_path(&id);
            let json = read_json(&path).await?;
            Ok(json)
        }

        async fn update(
            &mut self,
            id: String,
            partial_data: serde_json::Value,
        ) -> Result<(), Box<dyn Error>> {
            let partial_data = partial_data
                .as_object()
                .ok_or("Partial data is not an object")?;

            let json_before = self.get(id).await?;
            let json_before = json_before.as_object().ok_or("Not an object")?;

            let mut json_after = json_before.clone();

            for (key, value) in partial_data {
                json_after.insert(key.to_string(), value.clone());
            }

            Ok(())
        }

        async fn delete(&mut self, id: String) -> Result<(), Box<dyn Error>> {
            todo!()
        }
    }

    pub struct RepositoryPlayer {
        address_owner: String,
        name: String,
    }
}
