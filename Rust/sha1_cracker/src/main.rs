use sha1::Digest;

use std::{env, error::Error, fs};

const SHA1_HEX_STRING_LENGTH: usize = 40;

fn main() -> Result<(), Box<dyn Error>> {
    let cmd_args: Vec<String> = env::args().collect();

    if cmd_args.len() != 3 {
        println!(
            "Usage: 
     sha1_craker: <wordlist.txt> <sha1_hash>"
        );
        return Ok(());
    }

    let hash_to_crack = cmd_args[2].trim();
    if hash_to_crack.len() != SHA1_HEX_STRING_LENGTH {
        return Err("sha1 hash is not valid".into());
    }

    let wordlist_file = fs::read_to_string(&cmd_args[1])?;

    for line in wordlist_file.split('\n') {
        let common_password = line.trim();

        if hash_to_crack == hex::encode(sha1::Sha1::digest(common_password.as_bytes())) {
            println!("Password found: {}", &common_password);
            return Ok(());
        }
    }
    println!("Password not found in {}", cmd_args[1]);

    Ok(())
}
