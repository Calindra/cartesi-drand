pub mod random {
    use std::ops::Range;

    use rand::prelude::*;
    use rand_pcg::Pcg64;
    use rand_seeder::Seeder;

    pub struct Random {
        seed: String,
    }

    impl Random {
        pub fn new(seed: String) -> Self {
            Random { seed }
        }

        pub fn generate_random_seed(&self, range: Range<usize>) -> usize {
            let mut rng: Pcg64 = Seeder::from(self.seed.clone()).make_rng();
            rng.gen_range(range)
        }
    }
}
