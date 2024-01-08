use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use clap::Parser;
use byteorder::{LittleEndian, ReadBytesExt};

fn readlen(f: &mut File, len: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = vec![0u8; len];
    f.read_exact(&mut buf)?;
    Ok(buf)
}

fn readstr(src: &mut File) -> Result<String, Box<dyn Error>> {
    let mut ret: String = String::with_capacity(256);
    for _ in 0..256 { // 256 is a good length limit, right
        let asciidiot = readlen(src, 1)?[0];
        if asciidiot == 0 {
            ret.shrink_to_fit();
            break
        }
        let test = char::from_u32(asciidiot as u32);
        ret.push(test.expect("found eof"));
    }
    Ok(ret)
}

struct RndEntry {
    filetype: String,
    filename: String,
    unk: bool
}

impl RndEntry {
    pub fn new() -> Self {
        Self {
            filetype: "".to_string(),
            filename: "".to_string(),
            unk: false,
        }
    }

    pub fn load(&mut self, src: &mut File) -> Result<(), Box<dyn Error>> {
        self.filetype = readstr(src)?;
        self.filename = readstr(src)?;
        self.unk = readlen(src, 1)?[0] != 0;
        Ok(())
    }
}

struct RndFile {
    version: u32,
    entry_ct: u32,
    entries: Vec<RndEntry>,
    files: Vec<u8>, // split via 0xDEADDEAD BE
}

impl RndFile {
    pub fn new() -> Self {
        Self {
            version: 0,
            entry_ct: 0,
            entries: vec![],
            files: vec![],
        }
    }

    pub fn load(&mut self, src: &mut File) -> Result<(), Box<dyn Error>> {
        self.version = src.read_u32::<LittleEndian>()?;
        self.entry_ct = src.read_u32::<LittleEndian>()?;
        for _ in 0 .. self.entry_ct {
            let mut entry = RndEntry::new();
            let _ = entry.load(src)?;
            println!("New entry of type {} named {} with unk {}", entry.filetype, entry.filename, entry.unk);
            self.entries.push(entry);
        }
        let _ = src.read(self.files.as_mut_slice())?;
        Ok(())
    }

    pub fn export(&mut self, dump: &mut PathBuf) -> Result<(), Box<dyn Error>> {
        let mut sliced_filestack = self.files.as_mut_slice();

        Ok(())
    }
}

#[derive(clap::Parser)]
struct Args {
    input_rnd: PathBuf,
    #[arg(short, required = false)]
    output_dir: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    println!("input file: {}", args.input_rnd.display());
    let mut rnd = RndFile::new();
    let in_ext: &str = args.input_rnd.extension().unwrap().to_str().unwrap();
    if in_ext == "gz" {
        // TODO un-gzip gzipped rnds
    } else if in_ext == "rnd" {
        let mut input_file = File::open(&args.input_rnd)?;
        let _ = rnd.load(&mut input_file)?;
    }

    Ok(())
}
