use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let payload_path = Path::new("src/payload.bin");
    if !payload_path.exists() {
        let mut file = File::create(payload_path).unwrap();
        file.write_all(&[0; 1024]).unwrap(); // Dummy 1KB payload
    }

    let key_path = Path::new("src/key.bin");
    if !key_path.exists() {
        let mut file = File::create(key_path).unwrap();
        file.write_all(&[0; 32]).unwrap(); // Dummy 32-byte key
    }
}

//
