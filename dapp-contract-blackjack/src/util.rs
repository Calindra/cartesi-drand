pub mod random {
    use rand::prelude::*;
    use rand_pcg::Pcg64;
    use rand_seeder::Seeder;

    pub fn generate_random_seed(seed: String) -> usize {
        let mut rng: Pcg64 = Seeder::from(seed).make_rng();
        rng.gen_range(0..51)
    }
}
