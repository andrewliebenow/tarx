mod include_libforeign;

use crate::foreign::include_libforeign::{
    ConvertRarToTar, DecompressBzipTwo, FreePointerAndLength, FAILURE_CODE, SUCCESS_CODE,
};
use std::{os::raw::c_void, slice};

type IncludeLibforeignPointerAndLength = crate::foreign::include_libforeign::pointer_and_length;

pub fn convert_rar_to_tar(
    mut input: Box<[u8]>,
    password: Option<String>,
) -> anyhow::Result<Box<[u8]>> {
    let convert_rar_to_tar_return_type = {
        let pointer_and_length = box_to_raw(&mut input)?;

        // The password or an empty `String`
        let password_to_use_string = password.unwrap_or_default();

        let mut password_to_use_box = password_to_use_string.into_bytes().into_boxed_slice();

        let password_pointer_and_length = box_to_raw(&mut password_to_use_box)?;

        let co = unsafe { ConvertRarToTar(pointer_and_length, password_pointer_and_length) };

        // TODO
        // Use `PhantomData` to encode lifetime requirements
        // Ensure that this data lives through the FFI call, but no further
        drop(input);
        drop(password_to_use_box);

        co
    };

    let data_box = raw_to_box(convert_rar_to_tar_return_type.c_data)?;
    let error_message_box = raw_to_box(convert_rar_to_tar_return_type.b_error_message)?;

    let status_code = convert_rar_to_tar_return_type.a_status_code;

    let status_code_u_three_two = u32::from(status_code);

    if status_code_u_three_two == SUCCESS_CODE {
        Ok(data_box)
    } else if status_code_u_three_two == FAILURE_CODE {
        let error_message_str = std::str::from_utf8(&error_message_box)?;

        anyhow::bail!(
            "Foreign function failed with status code {status_code_u_three_two}: \"{error_message_str}\""
        );
    } else {
        anyhow::bail!("Invalid foreign function status code {status_code_u_three_two} encountered");
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn decompress_bzip_two(mut input: Box<[u8]>) -> anyhow::Result<Box<[u8]>> {
    let decompress_bzip_two_return_type = {
        let pointer_and_length = box_to_raw(&mut input)?;

        let de = unsafe { DecompressBzipTwo(pointer_and_length) };

        // TODO
        // Use `PhantomData` to encode lifetime requirements
        // Ensure that this data lives through the FFI call, but no further
        drop(input);

        de
    };

    let data_box = raw_to_box(decompress_bzip_two_return_type.c_data)?;
    let error_message_box = raw_to_box(decompress_bzip_two_return_type.b_error_message)?;

    let status_code = decompress_bzip_two_return_type.a_status_code;

    let status_code_u_three_two = u32::from(status_code);

    if status_code_u_three_two == SUCCESS_CODE {
        Ok(data_box)
    } else if status_code_u_three_two == FAILURE_CODE {
        let error_message_str = std::str::from_utf8(&error_message_box)?;

        anyhow::bail!(
            "Foreign function failed with status code {status_code_u_three_two}: \"{error_message_str}\""
        );
    } else {
        anyhow::bail!("Invalid foreign function status code {status_code_u_three_two} encountered");
    }
}

// TODO
// Prevent double free
fn raw_to_box(pointer_and_length: IncludeLibforeignPointerAndLength) -> anyhow::Result<Box<[u8]>> {
    let result = match usize::try_from(pointer_and_length.b_length) {
        Ok(us) => {
            let pointer = pointer_and_length.a_pointer;

            let pointer_cast = pointer.cast::<u8>();

            let slice = unsafe { slice::from_raw_parts(pointer_cast, us) };

            // https://github.com/rust-lang/rust/issues/61980#issuecomment-503829869
            Ok(Box::<[u8]>::from(slice))
        }
        Err(tr) => {
            // TODO
            Err(anyhow::anyhow!(tr))
        }
    };

    unsafe {
        FreePointerAndLength(pointer_and_length);
    }

    result
}

fn box_to_raw(box_x: &mut Box<[u8]>) -> anyhow::Result<IncludeLibforeignPointerAndLength> {
    let len = box_x.len();

    let length = i32::try_from(len)?;

    let box_as_mut_ptr = box_x.as_mut_ptr();

    let pointer = box_as_mut_ptr.cast::<c_void>();

    Ok(IncludeLibforeignPointerAndLength {
        a_pointer: pointer,
        b_length: length,
    })
}
