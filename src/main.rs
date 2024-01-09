use std::{error::Error, fs};
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

#[derive(Clone)]
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
        let x = src.read_to_end(&mut self.files)?;
        if x == 0 {
            println!("no files?");
        }
        Ok(())
    }

    pub fn export(&mut self, dump: &PathBuf) -> Result<(), Box<dyn Error>> {
        let sliced_filestack = self.files.as_mut_slice();
        let mut offsets = vec![0usize];
        let mut files_vecs: Vec<Vec<u8>> = vec![vec![]];
        /*for i in 0..self.entry_ct {
            match files_windows.position(|delim| delim == [0xADu8, 0xDE, 0xAD, 0xDE]) {
                Some(x) => {
                    offsets.push(x); 
                    println!("new idiot found sized {x}");
                    files_vecs.push(vec![0; x]);
                    let mut f = dump.clone();
                    f.push(&self.entries[i as usize].filename);
                    fs::write(f, files_vecs[i as usize].as_slice())?;
                },
                None => continue
            }
        }*/
        let mut prevpos: usize = 0;
        for (position, arr) in sliced_filestack.windows(4).enumerate() {
            if arr.starts_with(&[0xAD, 0xDE, 0xAD, 0xDE]) {
                offsets.push(position + 4);
                println!("file ending @ {}", position);
                files_vecs.push(sliced_filestack[prevpos..position].to_vec());
                prevpos = position + 4;
            }
        }
        if files_vecs.len() - 1 != self.entry_ct as usize {
            panic!("FUCK (found files doesn't match listed files)");
        }
        //let mut filenames: Vec<PathBuf> = vec![];
        for i in 0 .. self.entry_ct as usize {
            let e = &self.entries.clone()[i];
            let mut f = dump.clone();
            if e.filename.to_string().contains(|c| c == '.') {
                f.push(e.filename.clone());
            } else {
                f.push(e.filename.clone() + "." + &e.filetype.to_lowercase());
            }
            println!("the idiot is going to {}", f.display());
            //filenames.push(f);
            fs::write(f, files_vecs[i].as_slice())?;
        }
        Ok(())
    }
}

#[derive(clap::Parser)]
struct Args {
    input_rnd: PathBuf,
    #[arg(short)]
    output_dir: Option<PathBuf>,
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
        let mut outdir;
        match args.output_dir.clone() {
            Some(_) => outdir = args.output_dir.unwrap(),
            None => {
                outdir = PathBuf::new();
                outdir.push(args.input_rnd.parent().unwrap().to_path_buf());
                outdir.push("_".to_owned() + args.input_rnd.file_stem().unwrap().to_str().unwrap())
            }
        }
        if !outdir.is_dir() {
            fs::create_dir(&outdir)?;
        }
        rnd.export(&outdir)?;
    }
    Ok(())
}
