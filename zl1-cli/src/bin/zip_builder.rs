use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn zip_dir(src_dir: &Path, dst_file: &Path) -> std::io::Result<()> {
    let file = File::create(dst_file)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for entry in walkdir(src_dir)? {
        let path = entry.as_path();
        let name = path.strip_prefix(src_dir).unwrap().to_str().unwrap().replace('\\', "/");
        if path.is_file() {
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn walkdir(dir: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            files.extend(walkdir(&p)?);
        } else {
            files.push(p);
        }
    }
    Ok(files)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: zip_builder <src_dir> <dst_zip>");
        return;
    }
    zip_dir(Path::new(&args[1]), Path::new(&args[2])).unwrap();
    println!("Packaged APK zip successfully!");
}
