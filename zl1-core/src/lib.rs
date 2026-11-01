use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crc32fast::Hasher;
use byteorder::{LittleEndian, WriteBytesExt};

pub const ZL1_MAGIC: &[u8; 4] = b"ZL1\0";
pub const ZL1_VERSION: u8 = 1;

pub const FLAG_SOLID: u8 = 1 << 0;
pub const FLAG_DEDUP: u8 = 1 << 1;
pub const FLAG_BCJ: u8 = 1 << 2;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileEntry {
    pub relative_path: String,
    pub original_size: u64,
    pub offset_in_stream: u64,
    pub crc32: u32,
    pub is_dir: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArchiveManifest {
    pub files: Vec<FileEntry>,
    pub total_uncompressed_size: u64,
    pub solid_stream_compressed_size: u64,
}

pub struct Compressor;

impl Compressor {
    /// Упаковывает каталог или файл в гибридный .zl1 архив со стандартной ZIP-совместимостью
    pub fn compress_to_archive<P: AsRef<Path>, Q: AsRef<Path>>(
        src_path: P,
        dst_archive: Q,
        apply_bcj: bool,
    ) -> io::Result<ArchiveManifest> {
        let src = src_path.as_ref();
        let mut file_entries = Vec::new();
        let mut raw_stream = Vec::new();

        if src.is_file() {
            let file_name = src.file_name().unwrap_or_default().to_string_lossy().to_string();
            let mut f = File::open(src)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            
            let mut hasher = Hasher::new();
            hasher.update(&buf);
            let crc = hasher.finalize();

            file_entries.push(FileEntry {
                relative_path: file_name,
                original_size: buf.len() as u64,
                offset_in_stream: 0,
                crc32: crc,
                is_dir: false,
            });
            raw_stream.extend_from_slice(&buf);
        } else if src.is_dir() {
            Self::collect_dir_entries(src, src, &mut file_entries, &mut raw_stream)?;
        }

        let total_uncompressed = raw_stream.len() as u64;

        // Сверхплотная компрессия потока через LZMA (Solid Ultra)
        let mut compressed_data = Vec::new();
        {
            let mut cursor_in = Cursor::new(&raw_stream);
            lzma_rs::lzma_compress(&mut cursor_in, &mut compressed_data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("LZMA compression error: {:?}", e)))?;
        }

        let compressed_size = compressed_data.len() as u64;

        let manifest = ArchiveManifest {
            files: file_entries,
            total_uncompressed_size: total_uncompressed,
            solid_stream_compressed_size: compressed_size,
        };
        let manifest_bytes = serde_json::to_vec(&manifest)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Формируем полезную нагрузку ZL1
        let mut zl1_payload = Vec::new();
        zl1_payload.write_all(ZL1_MAGIC)?;
        zl1_payload.write_all(&[ZL1_VERSION, FLAG_SOLID, 0x02, 0x00])?;
        zl1_payload.write_all(&total_uncompressed.to_le_bytes())?;
        let metadata_offset = 4 + 4 + 8 + 8 + compressed_size;
        zl1_payload.write_all(&metadata_offset.to_le_bytes())?;
        zl1_payload.write_all(&compressed_data)?;
        let manifest_len = manifest_bytes.len() as u64;
        zl1_payload.write_all(&manifest_len.to_le_bytes())?;
        zl1_payload.write_all(&manifest_bytes)?;
        let mut m_hasher = Hasher::new();
        m_hasher.update(&manifest_bytes);
        zl1_payload.write_all(&m_hasher.finalize().to_le_bytes())?;

        // Запись в гибридный контейнер (ZIP Local File Header + ZL1 Payload + ZIP Central Directory)
        // Любой стандартный ZIP-архиватор (WinRAR, 7-Zip, Проводник Windows, Total Commander) увидит архив как ZIP!
        let mut out = File::create(dst_archive)?;
        let entry_name = b"content.zl1";
        let mut crc_payload = Hasher::new();
        crc_payload.update(&zl1_payload);
        let payload_crc = crc_payload.finalize();
        let payload_len = zl1_payload.len() as u32;

        // 1. ZIP Local File Header (0x04034b50)
        out.write_u32::<LittleEndian>(0x04034b50)?;
        out.write_u16::<LittleEndian>(20)?; // version needed
        out.write_u16::<LittleEndian>(0)?;  // flags
        out.write_u16::<LittleEndian>(0)?;  // method 0 (Stored, raw solid stream)
        out.write_u16::<LittleEndian>(0)?;  // mod time
        out.write_u16::<LittleEndian>(0)?;  // mod date
        out.write_u32::<LittleEndian>(payload_crc)?; // CRC32
        out.write_u32::<LittleEndian>(payload_len)?; // compressed size
        out.write_u32::<LittleEndian>(payload_len)?; // uncompressed size
        out.write_u16::<LittleEndian>(entry_name.len() as u16)?; // name length
        out.write_u16::<LittleEndian>(0)?;  // extra length
        out.write_all(entry_name)?;

        // Данные ZL1
        out.write_all(&zl1_payload)?;

        // 2. ZIP Central Directory (0x02014b50)
        let cd_offset = 30 + entry_name.len() as u32 + payload_len;
        out.write_u32::<LittleEndian>(0x02014b50)?;
        out.write_u16::<LittleEndian>(20)?; // version made by
        out.write_u16::<LittleEndian>(20)?; // version needed
        out.write_u16::<LittleEndian>(0)?;  // flags
        out.write_u16::<LittleEndian>(0)?;  // method
        out.write_u16::<LittleEndian>(0)?;  // mod time
        out.write_u16::<LittleEndian>(0)?;  // mod date
        out.write_u32::<LittleEndian>(payload_crc)?;
        out.write_u32::<LittleEndian>(payload_len)?;
        out.write_u32::<LittleEndian>(payload_len)?;
        out.write_u16::<LittleEndian>(entry_name.len() as u16)?;
        out.write_u16::<LittleEndian>(0)?; // extra length
        out.write_u16::<LittleEndian>(0)?; // comment length
        out.write_u16::<LittleEndian>(0)?; // disk num
        out.write_u16::<LittleEndian>(0)?; // internal attr
        out.write_u32::<LittleEndian>(0)?; // external attr
        out.write_u32::<LittleEndian>(0)?; // relative offset of local header
        out.write_all(entry_name)?;

        // 3. ZIP End of Central Directory (0x06054b50)
        let cd_size = 46 + entry_name.len() as u32;
        out.write_u32::<LittleEndian>(0x06054b50)?;
        out.write_u16::<LittleEndian>(0)?; // disk number
        out.write_u16::<LittleEndian>(0)?; // disk cd start
        out.write_u16::<LittleEndian>(1)?; // total entries disk
        out.write_u16::<LittleEndian>(1)?; // total entries
        out.write_u32::<LittleEndian>(cd_size)?; // cd size
        out.write_u32::<LittleEndian>(cd_offset)?; // offset of cd
        out.write_u16::<LittleEndian>(0)?; // comment len

        Ok(manifest)
    }

    fn collect_dir_entries(
        base_dir: &Path,
        current_dir: &Path,
        entries: &mut Vec<FileEntry>,
        stream: &mut Vec<u8>,
    ) -> io::Result<()> {
        let read_dir = std::fs::read_dir(current_dir)?;
        let mut paths: Vec<PathBuf> = read_dir.filter_map(|r| r.ok().map(|e| e.path())).collect();
        paths.sort();

        for path in paths {
            let rel = path.strip_prefix(base_dir).unwrap_or(&path).to_string_lossy().replace('\\', "/");
            if path.is_dir() {
                entries.push(FileEntry {
                    relative_path: rel,
                    original_size: 0,
                    offset_in_stream: stream.len() as u64,
                    crc32: 0,
                    is_dir: true,
                });
                Self::collect_dir_entries(base_dir, &path, entries, stream)?;
            } else if path.is_file() {
                let mut f = File::open(&path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;

                let mut hasher = Hasher::new();
                hasher.update(&buf);
                let crc = hasher.finalize();

                entries.push(FileEntry {
                    relative_path: rel,
                    original_size: buf.len() as u64,
                    offset_in_stream: stream.len() as u64,
                    crc32: crc,
                    is_dir: false,
                });
                stream.extend_from_slice(&buf);
            }
        }
        Ok(())
    }
}

pub struct Decompressor;

impl Decompressor {
    /// Извлекает архив .zl1 (прямой или гибридный ZIP-контейнер) в целевую директорию
    pub fn decompress_archive<P: AsRef<Path>, Q: AsRef<Path>>(
        archive_path: P,
        dest_dir: Q,
    ) -> io::Result<ArchiveManifest> {
        let mut file = File::open(archive_path)?;
        let mut header = [0u8; 4];
        file.read_exact(&mut header)?;

        let mut zl1_data = Vec::new();
        if &header == b"PK\x03\x04" {
            // Гибридный ZIP-контейнер: пропускаем ZIP-заголовок
            file.seek(SeekFrom::Start(26))?;
            let mut len_buf = [0u8; 2];
            file.read_exact(&mut len_buf)?;
            let name_len = u16::from_le_bytes(len_buf) as u64;
            file.read_exact(&mut len_buf)?;
            let extra_len = u16::from_le_bytes(len_buf) as u64;
            file.seek(SeekFrom::Start(30 + name_len + extra_len))?;
            file.read_to_end(&mut zl1_data)?;
        } else if &header == ZL1_MAGIC {
            // Прямой ZL1 архив
            zl1_data.extend_from_slice(&header);
            file.read_to_end(&mut zl1_data)?;
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported archive format"));
        }

        let mut cursor = Cursor::new(&zl1_data);
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        if &magic != ZL1_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid ZL1 payload signature"));
        }

        let mut flags_buf = [0u8; 4];
        cursor.read_exact(&mut flags_buf)?;
        let flags = flags_buf[1];

        let mut num_buf = [0u8; 8];
        cursor.read_exact(&mut num_buf)?;
        let _total_uncompressed = u64::from_le_bytes(num_buf);
        cursor.read_exact(&mut num_buf)?;
        let metadata_offset = u64::from_le_bytes(num_buf) as usize;

        // Извлекаем длину манифеста
        let mut len_buf = [0u8; 8];
        len_buf.copy_from_slice(&zl1_data[metadata_offset..metadata_offset + 8]);
        let manifest_len = u64::from_le_bytes(len_buf) as usize;

        // Извлекаем манифест
        let manifest_bytes = &zl1_data[metadata_offset + 8..metadata_offset + 8 + manifest_len];
        let manifest: ArchiveManifest = serde_json::from_slice(manifest_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Извлекаем сжатый поток
        let compressed_buf = &zl1_data[24..metadata_offset];
        let mut uncompressed_stream = Vec::new();
        {
            let mut lz_in = Cursor::new(compressed_buf);
            lzma_rs::lzma_decompress(&mut lz_in, &mut uncompressed_stream)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("LZMA error: {:?}", e)))?;
        }

        let out_dir = dest_dir.as_ref();
        std::fs::create_dir_all(out_dir)?;

        for entry in &manifest.files {
            let target_path = out_dir.join(&entry.relative_path);
            if entry.is_dir {
                std::fs::create_dir_all(&target_path)?;
            } else {
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let start = entry.offset_in_stream as usize;
                let end = start + entry.original_size as usize;
                let file_data = &uncompressed_stream[start..end];

                let mut out = File::create(&target_path)?;
                out.write_all(file_data)?;
            }
        }

        Ok(manifest)
    }
}
