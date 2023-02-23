macro_rules! sha256str {
    ($str:expr) => {
        {
            use sha2::{Sha256, Digest};

            let mut hasher = Sha256::new();
            hasher.update($str);
            let result = hasher.finalize();

            hex::encode(result)
        }
    };
}

macro_rules! regex_name {
    ($rgx:expr) => {
        {
            let mut hex = sha256str!($rgx);
            // Remove if clashing happens
            hex.truncate(hex.len() / 2);
            hex.insert_str(0, "__");
            hex
        }
    };
}