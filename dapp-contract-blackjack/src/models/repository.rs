pub mod repository {
    use std::error::Error;

    trait RepositoryCrud<T, I>
    where
        T: Clone + Default,
    {
        fn create(&mut self, data: T) -> Result<I, Box<dyn Error>>;
        fn get(&self, id: I) -> Result<T, Box<dyn Error>>;
        fn update(&mut self, partial_data: T) -> Result<(), Box<dyn Error>>;
        fn delete(&mut self, id: I) -> Result<(), Box<dyn Error>>;
    }


    struct RepositoryMemory {}
}
