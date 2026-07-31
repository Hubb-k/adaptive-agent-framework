use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use thiserror::Error;

const MAGIC: &[u8; 4] = b"AAGF";
const VERSION: u32 = 1;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
    #[error("Invalid magic bytes")]
    InvalidMagic,
    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },
}

pub struct BinarySerializer;

impl BinarySerializer {
    pub fn save<T: Serialize>(path: &str, data: &T) -> Result<(), StateError> {
        let bytes = bincode::serialize(data)?;

        let mut file = fs::File::create(path)?;
        use std::io::Write;
        file.write_all(MAGIC)?;
        file.write_all(&VERSION.to_le_bytes())?;
        file.write_all(&bytes)?;

        Ok(())
    }

    pub fn load<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, StateError> {
        let mut file = fs::File::open(path)?;
        use std::io::Read;

        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(StateError::InvalidMagic);
        }

        let mut version_buf = [0u8; 4];
        file.read_exact(&mut version_buf)?;
        let version = u32::from_le_bytes(version_buf);
        if version != VERSION {
            return Err(StateError::VersionMismatch {
                expected: VERSION,
                found: version,
            });
        }

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let data = bincode::deserialize(&bytes)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestState {
        tick: u64,
        value: f64,
        label: String,
    }

    #[test]
    fn test_save_load_roundtrip() {
        let path = "test_state.bin";
        let state = TestState {
            tick: 42,
            value: std::f64::consts::PI,
            label: "test".to_string(),
        };

        BinarySerializer::save(path, &state).unwrap();
        let loaded: TestState = BinarySerializer::load(path).unwrap();

        assert_eq!(state, loaded);
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_invalid_magic() {
        let path = "test_bad.bin";
        fs::write(path, b"XXXX").unwrap();
        let result: Result<TestState, _> = BinarySerializer::load(path);
        assert!(result.is_err());
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_complex_structures() {
        let path = "test_complex.bin";
        let data = vec![
            TestState {
                tick: 1,
                value: 0.5,
                label: "a".to_string(),
            },
            TestState {
                tick: 2,
                value: 0.7,
                label: "b".to_string(),
            },
        ];

        BinarySerializer::save(path, &data).unwrap();
        let loaded: Vec<TestState> = BinarySerializer::load(path).unwrap();

        assert_eq!(data, loaded);
        fs::remove_file(path).ok();
    }
}
