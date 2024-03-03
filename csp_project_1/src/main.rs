use clap::{Args, Parser, Subcommand, ValueEnum};
use rand::Rng;
use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Gen { size: usize, file: String },

    #[command(arg_required_else_help = true)]
    Run {
        num_threads: i32,
        num_hash_bits: i32,
        partitioning_method: i32,
    },
}

fn main() -> io::Result<()> {
    read_data("./test.data");
    let args = Cli::parse();
    match args.command {
        Commands::Gen { size, file } => gen_data(size, file.as_str()),
        Commands::Run {
            num_threads,
            num_hash_bits,
            partitioning_method,
        } => {
                println!(
                    "#threads {} #bits {} #part method{}",
                    num_threads, num_hash_bits, partitioning_method
                );
            match partitioning_method {
                1 => independent_output(num_threads, num_hash_bits),
                2 => count_then_move(num_threads, num_hash_bits),
                _ => panic!("Invalid partitioning method! Pls give 1 or 2"),
            };
            Ok(())
        }
    }?;


    //DATA GENERATION
    //we need to do it ourselves
    //and do it in a way where reading from disk
    //is NOT included in our measurements e.g.
    //generate just before experiement or make sure
    //that it is read into memory (not streamed) just before
    Ok(())
}

fn read_data(file_path: &str) -> Vec<(u64, u64)> {
    //file consisting of tuples of 8 byte partitioning key and 8 byte payload
    //could have used byteorder crate
    //but resolved to an answer by Alice Ryhl here: https://users.rust-lang.org/t/reading-binary-files-a-trivial-program-not-so-trivial-for-me/56166/3 
    let mut tuples: Vec<(u64, u64)> = Vec::new();
    let file = fs::read(file_path).unwrap();
    for bytes in file.chunks_exact(16) {
        let key =  u64::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]);
        let payload = u64::from_ne_bytes([bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]);
        //println!("{:x} {:x}", key, payload);
        tuples.push((key, payload));
    }
    tuples
}

fn hash(part_key: i64, hash_bits: i64) -> i64 {
    //partitioning key is 8 byte aka 64 bits
    part_key % 2 << hash_bits
}

fn independent_output(num_threads: i32, num_hash_bits: i32) {
    //coordination of input tuples to each thread is necessary
    //suggestion: divide input in num_threads blocks and assign

    //create output buffers for each thread, for each partition
    //t*(2^b) output buffers
    //where b is the number of hash bits
    //do hash key % hash bits
    //where hash bits is 1-18
    let N: i32 = 90000000; //placeholder -> replace with
    let buffer_size: i32 = N / (num_threads * (2 << num_hash_bits));
    let num_buffers: i32 = num_threads * (2 << num_hash_bits);
    let buffers: Vec<Vec<u64>> = vec![vec![0; buffer_size as usize]; num_buffers as usize];
}

fn count_then_move(num_threads: i32, num_hash_bits: i32) {}

fn gen_data(size: usize, file: &str) -> io::Result<()> {
    println!("Writing {} tuples to {}...", size, file);
    let mut rng = rand::thread_rng();
    let mut f = File::create(file)?;
    for i in 0..size {
        if i % 1000 == 0 {
            print!("Tuple: {} of {}            \r", i, size)
        }
        let key: u64 = i as u64;
        let val: u64 = rng.gen();
        f.write_all(&key.to_ne_bytes())?;
        f.write_all(&val.to_ne_bytes())?;
    }
    println!("Tuple: {} of {}           ", size, size);
    println!("Done!");

    Ok(())
}
