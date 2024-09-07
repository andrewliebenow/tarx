#![deny(clippy::all)]
#![warn(clippy::pedantic)]

#[cfg(feature = "foreign")]
mod foreign;

use anyhow::Context;
use clap::Parser;
use flate2::read::GzDecoder;
use std::{
    env,
    fs::{self, File, Permissions},
    io::{self, BufReader, Cursor, Write},
    os::unix::fs::MetadataExt,
    path::{self, PathBuf},
};
use tar::Archive;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use zip::{read::ZipFile, result::ZipError, ZipArchive};

#[cfg(feature = "dlmalloc")]
#[global_allocator]
static GLOBAL_DLMALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

/// Extract a .7z, .rar, .tar, .tar.bz2, .tar.gz, .tar.xz, or .zip file to a new directory
#[derive(Parser)]
#[command(author, version, about)]
struct TarxArgs {
    /// Password of the encrypted archive file to be extracted
    #[arg(long = "password", short = 'p')]
    password: Option<String>,

    /// Interactively enter the password of the encrypted archive file to be extracted
    #[arg(long = "type-password", short = 't')]
    type_password: bool,

    /// Path of the archive file to be extracted
    #[arg(index = 1_usize)]
    archive_file_path: String,
}

const DOT_RAR: &str = ".rar";
const DOT_SEVEN_Z: &str = ".7z";
const DOT_TAR_BZ_TWO: &str = ".tar.bz2";
const DOT_TAR_GZ: &str = ".tar.gz";
const DOT_TAR_XZ: &str = ".tar.xz";
const DOT_TAR: &str = ".tar";
const DOT_TGZ: &str = ".tgz";
const DOT_ZIP: &str = ".zip";

enum FileType {
    Rar,
    SevenZ,
    Tar,
    TarBzTwo,
    TarGz,
    TarXz,
    Zip,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), i32> {
    // TODO
    env::set_var("RUST_BACKTRACE", "1");
    // TODO
    env::set_var("RUST_LOG", "debug");

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let result = start();

    if let Err(er) = result {
        tracing::error!(
            backtrace = %er.backtrace(),
            error = %er,
        );

        return Err(1_i32);
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn start() -> anyhow::Result<()> {
    let TarxArgs {
        archive_file_path,
        password,
        type_password,
    } = TarxArgs::parse();

    let path = path::Path::new(&archive_file_path);

    let path_buf = fs::canonicalize(path)?;

    anyhow::ensure!(
        !path_buf.is_dir(),
        format!(
            "\"{}\" points to a directory, but it needs to point to a file",
            nameof::name_of!(path_buf)
        )
    );

    let file_name_os_str = path_buf.file_name().context("Could not get file name")?;

    let file_name_str = file_name_os_str.to_str().context(format!(
        "\"{}\" is not a valid UTF-8 string",
        nameof::name_of!(file_name_os_str)
    ))?;

    let file_name_str_ascii_lower_case = file_name_str.to_ascii_lowercase();

    let file_name_str_ascii_lower_case_before_period_stripped = file_name_str_ascii_lower_case
        .chars()
        .skip_while(|ch| !matches!(ch, '.'))
        .collect::<String>();

    anyhow::ensure!(
        !file_name_str_ascii_lower_case_before_period_stripped.is_empty(),
        "Only files with extensions are supported"
    );

    let (extension, file_type) = match &file_name_str_ascii_lower_case_before_period_stripped {
        st if st.ends_with(DOT_RAR) => (DOT_RAR, FileType::Rar),
        st if st.ends_with(DOT_SEVEN_Z) => (DOT_SEVEN_Z, FileType::SevenZ),
        st if st.ends_with(DOT_TAR_BZ_TWO) => (DOT_TAR_BZ_TWO, FileType::TarBzTwo),
        st if st.ends_with(DOT_TAR_GZ) => (DOT_TAR_GZ, FileType::TarGz),
        st if st.ends_with(DOT_TAR_XZ) => (DOT_TAR_XZ, FileType::TarXz),
        st if st.ends_with(DOT_TAR) => (DOT_TAR, FileType::Tar),
        st if st.ends_with(DOT_TGZ) => (DOT_TGZ, FileType::TarGz),
        st if st.ends_with(DOT_ZIP) => (DOT_ZIP, FileType::Zip),
        _ => {
            anyhow::bail!("Unrecognized file extension");
        }
    };

    let password_to_use = match file_type {
        FileType::Rar | FileType::SevenZ | FileType::Zip =>
            match (password, type_password) {
                // No password
                (None, false) => None,
                // Typed password
                (None, true) => {
                    io
                        ::stdout()
                        .write_all(
                            b"Password (note that the terminal will be cleared after a password is entered):\n"
                        )?;

                    let mut read_line = String::with_capacity(256_usize);

                    io::stdin().read_line(&mut read_line)?;

                    // Clear the terminal to hide the entered password
                    // https://stackoverflow.com/questions/34837011/how-to-clear-the-terminal-screen-in-rust-after-a-new-line-is-printed/34837038#34837038
                    io::stdout().write_all(&[27_u8, b'[', b'2', b'J'])?;

                    let op = read_line.pop();

                    if op != Some('\n') {
                        anyhow::bail!("Typed password input does not end with a newline character");
                    }

                    let opt = read_line.pop();

                    if let Some(ch) = opt {
                        if ch == '\r' {
                            tracing::debug!(
                                "Encountered and trimmed a carriage return character at the end of the typed password input. If the password you entered ends with a carriage return character, try using the \"--password\"/\"-p\" option instead."
                            );
                        } else {
                            read_line.push(ch);
                        }
                    }

                    for ch in read_line.chars() {
                        if ch == '\n' || ch == '\r' {
                            anyhow::bail!(
                                "Typed password input contains a carriage return or newline character. Passwords containing these characters are not supported via the typed password input option. Try using the \"--password\"/\"-p\" option instead."
                            );
                        }
                    }

                    Some(read_line)
                }
                // Passed password
                (Some(st), false) => Some(st),
                // Invalid
                (Some(_), true) =>
                    anyhow::bail!(
                        "\"--password\"/\"-p\" and \"--type-password\"/\"-t\" cannot be used at the same time"
                    ),
            }
        _ => {
            match (password, type_password) {
                // No password
                (None, false) => None,
                // Invalid
                _ => {
                    anyhow::bail!(
                        "Encryption is only supported for .7z, .rar, and .zip files. Remove the \"--password\"/\"-p\" option and/or the \"--type-password\"/\"-t\" option."
                    );
                }
            }
        }
    };

    let new_directory = get_new_directory(file_name_str, extension)?;

    let path_buf_file = File::open(&path_buf)?;

    match file_type {
        FileType::Rar => {
            #[cfg(feature = "foreign")]
            {
                use std::io::Read;

                tracing::warn!(
                    ".rar extraction uses FFI to Go code, and this integration is naive and all in-memory. Extraction will fail if your system does not have enough free memory to store the .rar file plus its decompressed contents."
                );

                // TODO
                // Performance: capacity
                let mut vec = Vec::new();

                let mut path_buf_file_mut = path_buf_file;

                path_buf_file_mut.read_to_end(&mut vec)?;

                let decompressed_box =
                    foreign::convert_rar_to_tar(vec.into_boxed_slice(), password_to_use)?;

                let cursor = Cursor::new(decompressed_box);

                let mut archive = Archive::new(cursor);

                archive.unpack(&new_directory)?;
            }

            #[cfg(not(feature = "foreign"))]
            {
                anyhow::bail!(
                    "Extracting .rar files requires Go to be installed and the \"foreign\" feature to be enabled"
                )
            }
        }
        FileType::SevenZ =>
        {
            #[allow(clippy::needless_borrowed_reference)]
            if let &Some(ref st) = &password_to_use {
                sevenz_rust::decompress_file_with_password(
                    &path_buf,
                    new_directory,
                    st.as_str().into(),
                )?;
            } else {
                sevenz_rust::decompress_file(path_buf, new_directory)?;
            }
        }
        FileType::Tar => {
            let mut archive = Archive::new(path_buf_file);

            archive.unpack(&new_directory)?;
        }
        FileType::TarBzTwo => {
            #[cfg(feature = "foreign")]
            {
                use std::io::Read;

                tracing::warn!(
                    ".tar.bz2 extraction uses FFI to Go code, and this integration is naive and all in-memory. Extraction will fail if your system does not have enough free memory to store the .tar.bz2 file plus the decompressed .tar file."
                );

                // TODO
                // Performance: capacity
                let mut vec = Vec::new();

                let mut path_buf_file_mut = path_buf_file;

                path_buf_file_mut.read_to_end(&mut vec)?;

                let decompressed_box = foreign::decompress_bzip_two(vec.into_boxed_slice())?;

                let cursor = Cursor::new(decompressed_box);

                let mut archive = Archive::new(cursor);

                archive.unpack(&new_directory)?;
            }

            #[cfg(not(feature = "foreign"))]
            {
                anyhow::bail!(
                    "Extracting .tar.bz2 files requires Go to be installed and the \"foreign\" feature to be enabled"
                )
            }
        }
        FileType::TarGz => {
            let gz_decoder = GzDecoder::new(path_buf_file);

            let mut archive = Archive::new(gz_decoder);

            archive.unpack(&new_directory)?;
        }
        FileType::TarXz => {
            let size = path_buf_file.metadata()?.size();

            let size_usize = usize::try_from(size)?;

            // TODO
            // Set capacity to some multiple of the file size
            let mut vec = Vec::with_capacity(size_usize);

            let mut buf_reader = BufReader::new(path_buf_file);

            lzma_rs::xz_decompress(&mut buf_reader, &mut vec)?;

            let cursor = Cursor::new(vec.into_boxed_slice());

            let mut archive = Archive::new(cursor);

            archive.unpack(&new_directory)?;
        }
        FileType::Zip => {
            // Adapted from https://github.com/zip-rs/zip2/blob/e3c81023a7ebedceaf287be98f3a10b5c1c18f8e/examples/extract.rs
            let mut zip_archive = ZipArchive::new(path_buf_file)?;

            #[allow(clippy::type_complexity)]
            let get_zip_file: Box<
                dyn for<'a> Fn(&'a mut ZipArchive<File>, usize) -> Result<ZipFile<'a>, ZipError>,
            > = if let Some(st) = password_to_use {
                let box_x = st.into_bytes().into_boxed_slice();

                Box::new(move |zip_archive: &mut ZipArchive<File>, index: usize| {
                    zip_archive.by_index_decrypt(index, &box_x)
                })
            } else {
                Box::new(|zip_archive: &mut ZipArchive<File>, index: usize| {
                    zip_archive.by_index(index)
                })
            };

            for us in 0_usize..zip_archive.len() {
                let mut zip_file = get_zip_file(&mut zip_archive, us)?;

                let enclosed_name = zip_file.enclosed_name();

                let Some(pa) = enclosed_name else {
                    tracing::warn!(?enclosed_name, "Could not get name of contained file");

                    continue;
                };

                let destination_path_buf = new_directory.join(pa);

                {
                    let comment = zip_file.comment();

                    if !comment.is_empty() {
                        tracing::info!("File {us} comment: \"{comment}\"");
                    }
                }

                if zip_file.is_dir() {
                    fs::create_dir_all(&destination_path_buf)?;
                } else {
                    if let Some(pat) = destination_path_buf.parent() {
                        if !pat.exists() {
                            fs::create_dir_all(pat)?;
                        }
                    }

                    let mut file = File::create(&destination_path_buf)?;

                    io::copy(&mut zip_file, &mut file)?;
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;

                    if let Some(ut) = zip_file.unix_mode() {
                        fs::set_permissions(&destination_path_buf, Permissions::from_mode(ut))?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_new_directory(file_name: &str, extension: &str) -> anyhow::Result<PathBuf> {
    let file_name_without_extension = strip_extension(file_name, extension)?;

    make_new_directory(file_name_without_extension)
}

fn strip_extension<'a>(file_name: &'a str, extension: &str) -> anyhow::Result<&'a str> {
    file_name
        .strip_suffix(extension)
        .context("Could not remove extension from file name")
}

fn make_new_directory(file_name_without_extension: &str) -> anyhow::Result<PathBuf> {
    let path_buf = env::current_dir()?;

    let new_directory_path_buf = path_buf.join(file_name_without_extension);

    fs::create_dir(&new_directory_path_buf)?;

    Ok(new_directory_path_buf)
}
