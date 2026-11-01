# Спецификация формата ZL1 (.zl1)

## 1. Заголовок (Header)
- `Magic Bytes`: 4 байта = `0x5A, 0x4C, 0x31, 0x00` (`"ZL1\0"`)
- `Version`: 1 байт (текущая версия `1`)
- `Flags`: 1 байт:
  - Bit 0: Solid compression
  - Bit 1: Deduplication enabled
  - Bit 2: BCJ filter applied
  - Bit 3: Encryption / Checksum type
- `Compression Algorithm`: 1 байт:
  - `0x01`: ZSTD (fast/ultra)
  - `0x02`: LZMA2 (ultra compact)
  - `0x03`: ZPAQ / Context model (extreme)
- `Block Size`: 4 байта (размер окна словаря / блока дедупликации)
- `File Count`: 4 байта (количество упакованных файлов)
- `Metadata Offset`: 8 байт (смещение до таблицы метаданных файлов в конце потока)
- `Header Checksum`: 4 байта (XXH3 / CRC32)

## 2. Поток данных (Data Chunks)
Последовательность сжатых блоков (Solid-поток или дедуплицированные чанки).
Каждый блок:
- `Chunk Header`: размер сжатого блока (4 байта), размер исходного блока (4 байта), CRC32 (4 байта).
- `Compressed Payload`.

## 3. Таблица метаданных файлов (Metadata Index)
Сжатый блок метаданных, содержащий:
- Список файлов:
  - Длина пути + путь (UTF-8)
  - Размер исходного файла (uint64)
  - Смещение в потоке распаковки (uint64)
  - Атрибуты / права доступа (unix/windows)
  - Контрольная сумма файла (CRC32/XXH3)
