use std::{env, fs, io, path, process};

use clap::Parser;

/// Extract a .7z, .rar, .tar.bz2, .tar.gz, .tar, or .zip file to a new directory
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct TarxArgs {
    /// Password of the encrypted archive file to be extracted
    #[arg(long = "password", short = 'p')]
    password: Option<String>,

    /// Interactively enter the password of the encrypted archive file to be extracted
    #[arg(long = "type-password", short = 't')]
    type_password: bool,

    /// Path of the archive file to be extracted
    #[arg(index = 1)]
    archive_file_path: String,
}

const DOT_RAR: &str = ".rar";
const DOT_SEVEN_Z: &str = ".7z";
const DOT_TAR_BZ_TWO: &str = ".tar.bz2";
const DOT_TAR_GZ: &str = ".tar.gz";
const DOT_TAR: &str = ".tar";
const DOT_ZIP: &str = ".zip";

enum FileType {
    Rar,
    SevenZ,
    Tar,
    TarBzTwo,
    TarGz,
    Zip,
}

fn main() {
    let TarxArgs {
        archive_file_path,
        password,
        type_password,
    } = TarxArgs::parse();

    let password_is_some = (&password).is_some();

    if password_is_some && type_password {
        panic!("\"--password\" and \"--type-password\" cannot be used at the same time");
    }

    let path = path::Path::new(&archive_file_path);

    let path_buf = fs::canonicalize(path).expect("Could not canonicalize archive file path");

    if path_buf.is_dir() {
        panic!("\"archive_file_path points\" to a directory, but it needs to point to a file");
    }

    let file_name_os_str = path_buf.file_name().expect("Could not get file name");

    let file_name_str = file_name_os_str
        .to_str()
        .expect("\"file_name_os_str\" is not a valid UTF-8 string");

    let file_name_str_ascii_lower_case = file_name_str.to_ascii_lowercase();

    let file_name_str_ascii_lower_case_before_period_stripped = file_name_str_ascii_lower_case
        .chars()
        .skip_while(|ch| match ch {
            '.' => false,
            _ => true,
        })
        .collect::<String>();

    if (&file_name_str_ascii_lower_case_before_period_stripped).is_empty() {
        panic!("Only files with extensions are supported");
    }

    let (extension, file_type) = match &file_name_str_ascii_lower_case_before_period_stripped {
        st if st.ends_with(DOT_RAR) => (DOT_RAR, FileType::Rar),
        st if st.ends_with(DOT_SEVEN_Z) => (DOT_SEVEN_Z, FileType::SevenZ),
        st if st.ends_with(DOT_TAR_BZ_TWO) => (DOT_TAR_BZ_TWO, FileType::TarBzTwo),
        st if st.ends_with(DOT_TAR_GZ) => (DOT_TAR_GZ, FileType::TarGz),
        st if st.ends_with(DOT_TAR) => (DOT_TAR, FileType::Tar),
        st if st.ends_with(DOT_ZIP) => (DOT_ZIP, FileType::Zip),
        _ => {
            panic!("Unrecognized file extension");
        }
    };

    let password_to_use = if password_is_some || type_password {
        match &file_type {
            FileType::Rar | FileType::SevenZ => {
                if type_password {
                    println!("Password:");

                    let mut read_line = String::new();

                    io::stdin()
                        .read_line(&mut read_line)
                        .expect("Could not read line from stdin");

                    Some(read_line)
                } else {
                    password
                }
            }
            _ => {
                panic!( "Encryption is only supported for .7z and .rar files. Remove the \"--password\"/\"-p\" or \"--type-password\"/\"-t\" option.");
            }
        }
    } else {
        None
    };

    let new_directory = get_new_directory(&file_name_str, &extension);

    match &file_type {
        FileType::Rar => {
            let mut command = process::Command::new("xt");

            command.arg("-output").arg(new_directory);

            if let Some(str) = &password_to_use {
                command.arg("-password").arg(str);
            }

            command
                .arg(path_buf)
                .stderr(process::Stdio::inherit())
                .stdout(process::Stdio::inherit())
                .output()
                .expect("Invocation of \"xt\" failed");
        }
        FileType::SevenZ => {
            if let Some(str) = &password_to_use {
                sevenz_rust::decompress_file_with_password(
                    path_buf,
                    new_directory,
                    str.as_str().into(),
                )
                .expect("7z extraction failed");
            } else {
                sevenz_rust::decompress_file(path_buf, new_directory)
                    .expect("7z extraction failed");
            }
        }
        FileType::TarBzTwo | FileType::TarGz | FileType::Tar => {
            process::Command::new("xt")
                .arg("-output")
                .arg(new_directory)
                .arg(path_buf)
                .stderr(process::Stdio::inherit())
                .stdout(process::Stdio::inherit())
                .output()
                .expect("Invocation of \"xt\" failed");
        }
        FileType::Zip => {
            process::Command::new("ripunzip")
                .arg("--output-directory")
                .arg(new_directory)
                .arg("file")
                .arg(path_buf)
                .stderr(process::Stdio::inherit())
                .stdout(process::Stdio::inherit())
                .output()
                .expect("Invocation of \"ripunzip\" failed");
        }
    }
}

fn get_new_directory(file_name: &str, extension: &str) -> path::PathBuf {
    let file_name_without_extension = strip_extension(&file_name, &extension);

    make_new_directory(&file_name_without_extension)
}

fn strip_extension<'a>(file_name: &'a str, extension: &str) -> &'a str {
    file_name
        .strip_suffix(extension)
        .expect("Could not remove extension from file name")
}

fn make_new_directory(file_name_without_extension: &str) -> path::PathBuf {
    let path_buf = env::current_dir().expect("Could not get current directory");

    let new_directory_path_buf = {
        let mut pa = path_buf.clone();

        pa.push(file_name_without_extension);

        pa
    };

    fs::create_dir(&new_directory_path_buf).expect("Could not create new directory");

    new_directory_path_buf
}
