use std::env;

const SHA1_HEX_STRING_LENGTH: usize = 40;

fn main() -> Result<(), &'static str> {
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
        return Err("sha1 hash is not valid");
    }

    Ok(())
}
