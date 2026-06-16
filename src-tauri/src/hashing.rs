use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

const HASH_BUFFER_SIZE: usize = 1024 * 1024;

pub fn compute_sha256(path: &Path) -> Result<String, String> {
    let file =
        File::open(path).map_err(|error| format!("Could not open file for hashing: {error}"))?;
    let mut reader = BufReader::with_capacity(HASH_BUFFER_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; HASH_BUFFER_SIZE];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|error| format!("Could not read file for hashing: {error}"))?;
        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::{compute_sha256, HASH_BUFFER_SIZE};
    use std::{fs, path::PathBuf};
    use uuid::Uuid;

    #[test]
    fn compute_sha256_hashes_empty_file() {
        let fixture = HashFixture::new();
        let path = fixture.write_file("empty.bin", b"");

        let hash = compute_sha256(&path).expect("empty file should hash");

        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn compute_sha256_hashes_known_content() {
        let fixture = HashFixture::new();
        let path = fixture.write_file("sample.txt", b"piTrace");

        let hash = compute_sha256(&path).expect("sample content should hash");

        assert_eq!(
            hash,
            "2c2a117128a031f56a6856f35f4dedb081b89f895e8e8fe8407e87acce432821"
        );
    }

    #[test]
    fn compute_sha256_streams_content_larger_than_buffer() {
        let fixture = HashFixture::new();
        let bytes = vec![b'a'; HASH_BUFFER_SIZE + 17];
        let path = fixture.write_file("large.bin", &bytes);

        let hash = compute_sha256(&path).expect("large file should hash");

        assert_eq!(
            hash,
            "c26032d5154f96bd29c799447d715ab681d8d0aa308ecc6f321a35d98f0672da"
        );
    }

    #[test]
    fn compute_sha256_reports_missing_file() {
        let fixture = HashFixture::new();
        let path = fixture.dir.join("missing.bin");

        let error = compute_sha256(&path).expect_err("missing file should fail");

        assert!(error.contains("Could not open file for hashing"));
    }

    #[test]
    fn compute_sha256_rejects_directory_path() {
        let fixture = HashFixture::new();
        let path = fixture.dir.join("directory.bin");
        fs::create_dir_all(&path).expect("directory should be created");

        let error = compute_sha256(&path).expect_err("directory should fail");

        assert!(
            error.contains("Could not open file for hashing")
                || error.contains("Could not read file for hashing")
        );
    }

    struct HashFixture {
        dir: PathBuf,
    }

    impl HashFixture {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("pi-trace-hash-test-{}", Uuid::new_v4()));
            fs::create_dir_all(&dir).expect("test directory should be created");

            Self { dir }
        }

        fn write_file(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.dir.join(name);
            fs::write(&path, bytes).expect("test file should be written");
            path
        }
    }

    impl Drop for HashFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
}
