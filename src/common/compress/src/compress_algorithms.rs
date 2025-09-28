// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

/// CompressAlgorithm represents all compress algorithm that OpenDAL supports.
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub enum CompressAlgorithm {
    /// [Brotli](https://github.com/google/brotli) compression format.
    Brotli,
    /// [bzip2](http://sourceware.org/bzip2/) compression format.
    Bz2,
    /// [Deflate](https://datatracker.ietf.org/doc/html/rfc1951) Compressed Data Format.
    ///
    /// Similar to [`CompressAlgorithm::Gzip`] and [`CompressAlgorithm::Zlib`]
    Deflate,
    /// [Gzip](https://datatracker.ietf.org/doc/html/rfc1952) compress format.
    ///
    /// Similar to [`CompressAlgorithm::Deflate`] and [`CompressAlgorithm::Zlib`]
    Gzip,
    /// [LZMA](https://www.7-zip.org/sdk.html) compress format.
    Lzma,
    /// [Xz](https://tukaani.org/xz/) compress format, the successor of [`CompressAlgorithm::Lzma`].
    Xz,
    /// [Zlib](https://datatracker.ietf.org/doc/html/rfc1950) compress format.
    ///
    /// Similar to [`CompressAlgorithm::Deflate`] and [`CompressAlgorithm::Gzip`]
    Zlib,
    /// [Zip](https://pkware.cachefly.net/webdocs/APPNOTE/APPNOTE-6.3.10.TXT) compress format.
    Zip,
    /// [Zstd](https://github.com/facebook/zstd) compression algorithm
    Zstd,
}

impl CompressAlgorithm {
    /// Get the file extension of this compress algorithm.
    pub fn extension(&self) -> &str {
        match self {
            CompressAlgorithm::Brotli => "br",
            CompressAlgorithm::Bz2 => "bz2",
            CompressAlgorithm::Deflate => "deflate",
            CompressAlgorithm::Gzip => "gz",
            CompressAlgorithm::Lzma => "lzma",
            CompressAlgorithm::Xz => "xz",
            CompressAlgorithm::Zlib => "zl",
            CompressAlgorithm::Zstd => "zstd",
            CompressAlgorithm::Zip => "zip",
        }
    }

    /// Create CompressAlgorithm from file extension.
    ///
    /// If the file extension is not supported, `None` will be return instead.
    pub fn from_extension(ext: &str) -> Option<CompressAlgorithm> {
        match ext {
            "br" => Some(CompressAlgorithm::Brotli),
            "bz2" => Some(CompressAlgorithm::Bz2),
            "deflate" => Some(CompressAlgorithm::Deflate),
            "gz" => Some(CompressAlgorithm::Gzip),
            "lzma" => Some(CompressAlgorithm::Lzma),
            "xz" => Some(CompressAlgorithm::Xz),
            "zl" => Some(CompressAlgorithm::Zlib),
            "zstd" | "zst" => Some(CompressAlgorithm::Zstd),
            "zip" => Some(CompressAlgorithm::Zip),
            _ => None,
        }
    }

    /// Create CompressAlgorithm from file path.
    ///
    /// If the extension in file path is not supported, `None` will be return instead.
    pub fn from_path(path: &str) -> Option<CompressAlgorithm> {
        let ext = PathBuf::from(path)
            .extension()
            .map(|s| s.to_string_lossy())?
            .to_string();

        CompressAlgorithm::from_extension(&ext)
    }

    /// Try to infer compression algorithm from the binary signature.
    ///
    /// This is best-effort detection so we only check the magic bytes for
    /// formats that have a well defined header. For the rest we return
    /// `None` so the caller can decide how to handle them.
    pub fn from_bytes(data: &[u8]) -> Option<CompressAlgorithm> {
        if data.len() >= 2 && data[0] == 0x1F && data[1] == 0x8B {
            return Some(CompressAlgorithm::Gzip);
        }

        if data.len() >= 4 && data[0..4] == [0x50, 0x4B, 0x03, 0x04] {
            return Some(CompressAlgorithm::Zip);
        }

        if data.len() >= 3 && &data[..3] == b"BZh" {
            return Some(CompressAlgorithm::Bz2);
        }

        if data.len() >= 6 && data[0..6] == [0xFD, b'7', b'z', b'X', b'Z', 0x00] {
            return Some(CompressAlgorithm::Xz);
        }

        if data.len() >= 4 && data[0..4] == [0x28, 0xB5, 0x2F, 0xFD] {
            return Some(CompressAlgorithm::Zstd);
        }

        if data.len() >= 2 {
            let cmf = data[0];
            let flg = data[1];
            if cmf == 0x78 && ((u16::from(cmf) << 8) + u16::from(flg)) % 31 == 0 {
                return Some(CompressAlgorithm::Zlib);
            }
        }

        None
    }

    /// Try to infer compression algorithm from either file path or binary signature.
    pub fn from_path_or_bytes(path: &str, data: &[u8]) -> Option<CompressAlgorithm> {
        CompressAlgorithm::from_path(path).or_else(|| CompressAlgorithm::from_bytes(data))
    }
}

#[cfg(test)]
mod tests {
    use super::CompressAlgorithm;

    #[test]
    fn test_from_bytes_gzip() {
        assert_eq!(
            CompressAlgorithm::from_bytes(&[0x1F, 0x8B, 0x08]),
            Some(CompressAlgorithm::Gzip)
        );
    }

    #[test]
    fn test_from_bytes_zip() {
        assert_eq!(
            CompressAlgorithm::from_bytes(&[0x50, 0x4B, 0x03, 0x04, 0x14]),
            Some(CompressAlgorithm::Zip)
        );
    }

    #[test]
    fn test_from_bytes_zstd() {
        assert_eq!(
            CompressAlgorithm::from_bytes(&[0x28, 0xB5, 0x2F, 0xFD, 0x00]),
            Some(CompressAlgorithm::Zstd)
        );
    }

    #[test]
    fn test_from_bytes_unknown() {
        assert_eq!(CompressAlgorithm::from_bytes(&[0u8; 10]), None);
    }
}
