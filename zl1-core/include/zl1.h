#ifndef ZL1_H
#define ZL1_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Compress a file or directory into a .zl1 archive.
 * @param src_path Path to input file or directory (UTF-8).
 * @param dst_archive Path to output .zl1 file.
 * @param apply_bcj Apply BCJ filter for x86/x64 executables.
 * @return 0 on success, negative value on error.
 */
int32_t zl1_compress(const char* src_path, const char* dst_archive, bool apply_bcj);

/**
 * Decompress a .zl1 archive into target directory.
 * @param archive_path Path to .zl1 archive.
 * @param dest_dir Path to destination folder.
 * @return 0 on success, negative value on error.
 */
int32_t zl1_decompress(const char* archive_path, const char* dest_dir);

#ifdef __cplusplus
}
#endif

#endif // ZL1_H
