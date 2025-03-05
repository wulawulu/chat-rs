use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

use super::ChatFile;

#[allow(dead_code)]
impl ChatFile {
    pub fn new(ws_id: u64, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ws_id,
            ext: filename.split('.').last().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self, ws_id: u64) -> String {
        format!("/file/{ws_id}/{}", self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}/{}.{}", self.ws_id, part1, part2, part3, self.ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_file_new_should_work() {
        let filename = "test.txt";
        let data = b"hello world";
        let chat_file = ChatFile::new(1, filename, data);
        assert_eq!(chat_file.ext, "txt");
        assert_eq!(chat_file.hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }
}
