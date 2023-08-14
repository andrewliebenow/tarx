use std::{env, fs, path, process};

use clap::Parser;

/// Extract a .7z, .rar, .tar.bz2, .tar.gz, .tar, or .zip file to a new directory
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct TarxArgs {
    /// Password of the encrypted archive file to be extracted
    #[arg(long = "password", short = 'p')]
    password: Option<String>,

    /// Path of the archive file to be extracted
    #[arg(index = 1)]
    archive_file_path: String,
}

const DOT_RAR: &str = ".rar";
const DOT_SEVEN_Z: &str = ".7z";
const DOT_TAR_BZ_TWO: &str = ".tar.bz2";
const DOT_TAR_GZ: &str = ".tar.gz";
const DOT_TAR_XZ: &str = ".tar.xz";
const DOT_TAR: &str = ".tar";
const DOT_ZIP: &str = ".zip";

fn main() {
    let tarx_args = TarxArgs::parse();

    let archive_file_path = &tarx_args.archive_file_path;
    let password = &tarx_args.password;

    let path = path::Path::new(archive_file_path);

    let path_buf = fs::canonicalize(path).expect("Could not canonicalize archive file path");

    if path_buf.is_dir() {
        panic!("\"archive_file_path points\" to a directory, but it needs to point to a file");
    }

    let file_name_os_str = path_buf.file_name().expect("Could not get file name");

    let file_name_str = file_name_os_str
        .to_str()
        .expect("\"file_name_os_str\" is not a valid UTF-8 string");

    let file_name_str_ascii_lower_case = file_name_str.to_ascii_lowercase();

    match file_name_str_ascii_lower_case {
        st if st.ends_with(DOT_SEVEN_Z) => {
            let new_directory = get_new_directory(file_name_str, DOT_SEVEN_Z);

            if let Some(str) = password {
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
        st if st.ends_with(DOT_RAR) => {
            let new_directory = get_new_directory(file_name_str, DOT_RAR);

            if let Some(str) = password {
                process::Command::new("unar")
                    .arg("-no-directory")
                    .arg("-output-directory")
                    .arg(new_directory)
                    .arg("-password")
                    .arg(str)
                    .arg(path_buf)
                    .stderr(process::Stdio::inherit())
                    .stdout(process::Stdio::inherit())
                    .output()
                    .expect("Invocation of \"unar\" failed");
            } else {
                process::Command::new("xt")
                    .arg("-output")
                    .arg(new_directory)
                    .arg(path_buf)
                    .stderr(process::Stdio::inherit())
                    .stdout(process::Stdio::inherit())
                    .output()
                    .expect("Invocation of \"xt\" failed");
            }
        }
        st if st.ends_with(DOT_TAR_BZ_TWO) => {
            handle_tar(password, &path_buf, file_name_str, DOT_TAR_BZ_TWO)
        }
        st if st.ends_with(DOT_TAR_GZ) => {
            handle_tar(password, &path_buf, file_name_str, DOT_TAR_GZ)
        }
        st if st.ends_with(DOT_TAR_XZ) => {
            unimplemented!();
        }
        st if st.ends_with(DOT_TAR) => handle_tar(password, &path_buf, file_name_str, DOT_TAR),
        st if st.ends_with(DOT_ZIP) => {
            panic_if_password_provided(password, DOT_ZIP);

            let new_directory = get_new_directory(file_name_str, DOT_ZIP);

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
        _ => {
            panic!("Unrecognized file extension");
        }
    }
}

fn panic_if_password_provided(password: &Option<String>, extension: &str) {
    if let Some(_) = password {
        panic!(
            "{} files do not support encryption. Remove the \"--password\" or \"-p\" option.",
            extension
        );
    }
}

fn handle_tar(
    password: &Option<String>,
    path_buf: &path::PathBuf,
    file_name_str: &str,
    extension: &str,
) {
    panic_if_password_provided(password, extension);

    let new_directory = get_new_directory(file_name_str, extension);

    process::Command::new("xt")
        .arg("-output")
        .arg(new_directory)
        .arg(path_buf)
        .stderr(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .output()
        .expect("Invocation of \"xt\" failed");
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
