use rcgen::generate_simple_self_signed;
use std::fs::File;
use std::io::Write;

fn main() {
    let subject_alt_names = vec!["nanodev.zl1".to_string(), "localhost".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).unwrap();

    let cert_pem = cert.cert.pem();
    let der = cert.signing_key.serialize_der();

    let mut cert_file = File::create("C:/Users/nanodev/.gemini/antigravity-cli/scratch/zl1-android-build/cert.pem").unwrap();
    cert_file.write_all(cert_pem.as_bytes()).unwrap();

    let mut key_file = File::create("C:/Users/nanodev/.gemini/antigravity-cli/scratch/zl1-android-build/key.pk8").unwrap();
    key_file.write_all(&der).unwrap();

    println!("Generated cert.pem and key.pk8 successfully!");
}
