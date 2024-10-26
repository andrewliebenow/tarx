mod include_libforeign;

use crate::foreign::include_libforeign::{FAILURE_CODE, SUCCESS_CODE};
use foreign_calls::{
    raw_to_box, safe_convert_rar_to_tar, safe_decompress_bzip_two, safe_decompress_zstd,
    SafeConvertRarToTarWrapperResult, SafeDecompressBzipTwoResult, SafeDecompressZstdResult,
};
use std::str;

pub fn convert_rar_to_tar(input: &mut [u8], password: Option<String>) -> anyhow::Result<Box<[u8]>> {
    let SafeConvertRarToTarWrapperResult {
        data,
        foreign_call_result,
    } = safe_convert_rar_to_tar(input, password)?;

    let data_box = raw_to_box(data)?;
    let error_message_box = raw_to_box(foreign_call_result.error_message)?;

    let status_code = foreign_call_result.status_code;

    let status_code_u_three_two = u32::from(status_code);

    if status_code_u_three_two == SUCCESS_CODE {
        Ok(data_box)
    } else if status_code_u_three_two == FAILURE_CODE {
        let error_message_str = str::from_utf8(&error_message_box)?;

        anyhow::bail!(
            "Foreign function failed with status code {status_code_u_three_two}: \"{error_message_str}\""
        );
    } else {
        anyhow::bail!("Invalid foreign function status code {status_code_u_three_two} encountered");
    }
}

// TODO
// Duplication
pub fn decompress_bzip_two(input: &mut [u8]) -> anyhow::Result<Box<[u8]>> {
    let SafeDecompressBzipTwoResult {
        data,
        foreign_call_result,
    } = safe_decompress_bzip_two(input)?;

    let data_box = raw_to_box(data)?;
    let error_message_box = raw_to_box(foreign_call_result.error_message)?;

    let status_code = foreign_call_result.status_code;

    let status_code_u_three_two = u32::from(status_code);

    if status_code_u_three_two == SUCCESS_CODE {
        Ok(data_box)
    } else if status_code_u_three_two == FAILURE_CODE {
        let error_message_str = str::from_utf8(&error_message_box)?;

        anyhow::bail!(
            "Foreign function failed with status code {status_code_u_three_two}: \"{error_message_str}\""
        );
    } else {
        anyhow::bail!("Invalid foreign function status code {status_code_u_three_two} encountered");
    }
}

// TODO
// Duplication
pub fn decompress_zstd(input: &mut [u8]) -> anyhow::Result<Box<[u8]>> {
    let SafeDecompressZstdResult {
        data,
        foreign_call_result,
    } = safe_decompress_zstd(input)?;

    let data_box = raw_to_box(data)?;
    let error_message_box = raw_to_box(foreign_call_result.error_message)?;

    let status_code = foreign_call_result.status_code;

    let status_code_u_three_two = u32::from(status_code);

    if status_code_u_three_two == SUCCESS_CODE {
        Ok(data_box)
    } else if status_code_u_three_two == FAILURE_CODE {
        let error_message_str = str::from_utf8(&error_message_box)?;

        anyhow::bail!(
            "Foreign function failed with status code {status_code_u_three_two}: \"{error_message_str}\""
        );
    } else {
        anyhow::bail!("Invalid foreign function status code {status_code_u_three_two} encountered");
    }
}

mod foreign_calls {
    use super::include_libforeign::{
        self, ConvertRarToTar, DecompressBzipTwo, DecompressZstd, FreePointerAndLength,
    };
    use std::{ffi::c_void, slice};

    type IncludeLibforeignPointerAndLength = include_libforeign::pointer_and_length;

    pub struct SafeConvertRarToTarWrapperResult {
        pub foreign_call_result: ForeignCallResult,
        pub data: ForeignAllocation,
    }

    pub struct SafeDecompressBzipTwoResult {
        pub foreign_call_result: ForeignCallResult,
        pub data: ForeignAllocation,
    }

    pub struct SafeDecompressZstdResult {
        pub foreign_call_result: ForeignCallResult,
        pub data: ForeignAllocation,
    }

    pub struct ForeignCallResult {
        pub error_message: ForeignAllocation,
        pub status_code: u8,
    }

    pub struct ForeignAllocation {
        pointer_and_length: IncludeLibforeignPointerAndLength,
    }

    impl Drop for ForeignAllocation {
        fn drop(&mut self) {
            // Safety: TODO, check with Miri
            unsafe {
                FreePointerAndLength(self.pointer_and_length);
            }
        }
    }

    impl ForeignAllocation {
        fn new(pointer_and_length: IncludeLibforeignPointerAndLength) -> ForeignAllocation {
            ForeignAllocation { pointer_and_length }
        }
    }

    pub fn safe_convert_rar_to_tar(
        input: &mut [u8],
        password: Option<String>,
    ) -> anyhow::Result<SafeConvertRarToTarWrapperResult> {
        let pointer_and_length = slice_to_raw(input)?;

        // The password or an empty `String`
        let password_to_use_string = password.unwrap_or_default();

        let mut password_to_use_vec = password_to_use_string.into_bytes();

        let password_pointer_and_length = slice_to_raw(&mut password_to_use_vec)?;

        // Safety: TODO, check with Miri
        let convert_rar_to_tar_return_type =
            unsafe { ConvertRarToTar(pointer_and_length, password_pointer_and_length) };

        let foreign_call_result = ForeignCallResult {
            error_message: ForeignAllocation::new(convert_rar_to_tar_return_type.b_error_message),
            status_code: convert_rar_to_tar_return_type.a_status_code,
        };

        // Ensure it is not dropped sooner
        drop(password_to_use_vec);

        Ok(SafeConvertRarToTarWrapperResult {
            data: ForeignAllocation::new(convert_rar_to_tar_return_type.c_data),
            foreign_call_result,
        })
    }

    pub fn safe_decompress_bzip_two(
        input: &mut [u8],
    ) -> anyhow::Result<SafeDecompressBzipTwoResult> {
        let pointer_and_length = slice_to_raw(input)?;

        // Safety: TODO, check with Miri
        let decompress_bzip_two_return_type = unsafe { DecompressBzipTwo(pointer_and_length) };

        let foreign_call_result = ForeignCallResult {
            error_message: ForeignAllocation::new(decompress_bzip_two_return_type.b_error_message),
            status_code: decompress_bzip_two_return_type.a_status_code,
        };

        Ok(SafeDecompressBzipTwoResult {
            foreign_call_result,
            data: ForeignAllocation::new(decompress_bzip_two_return_type.c_data),
        })
    }

    pub fn safe_decompress_zstd(input: &mut [u8]) -> anyhow::Result<SafeDecompressZstdResult> {
        let pointer_and_length = slice_to_raw(input)?;

        // Safety: TODO, check with Miri
        let decompress_zstd_return_type = unsafe { DecompressZstd(pointer_and_length) };

        let foreign_call_result = ForeignCallResult {
            error_message: ForeignAllocation::new(decompress_zstd_return_type.b_error_message),
            status_code: decompress_zstd_return_type.a_status_code,
        };

        Ok(SafeDecompressZstdResult {
            foreign_call_result,
            data: ForeignAllocation::new(decompress_zstd_return_type.c_data),
        })
    }

    /* #region Helper functions */
    fn slice_to_raw(slice: &mut [u8]) -> anyhow::Result<IncludeLibforeignPointerAndLength> {
        let len = slice.len();

        let length = i32::try_from(len)?;

        let slice_as_mut_ptr = slice.as_mut_ptr();

        let pointer = slice_as_mut_ptr.cast::<c_void>();

        Ok(IncludeLibforeignPointerAndLength {
            a_pointer: pointer,
            b_length: length,
        })
    }

    // Convert the allocation from a foreign allocation to a non-foreign allocation by copying the data
    pub fn raw_to_box(foreign_allocation: ForeignAllocation) -> anyhow::Result<Box<[u8]>> {
        let pointer_and_length = foreign_allocation.pointer_and_length;

        let length_usize = usize::try_from(pointer_and_length.b_length)?;

        let pointer = pointer_and_length.a_pointer;

        let pointer_cast = pointer.cast::<u8>();

        // Safety: TODO, check with Miri
        let slice = unsafe { slice::from_raw_parts(pointer_cast, length_usize) };

        // Immediately copy the data into a non-foreign allocation
        // https://github.com/rust-lang/rust/issues/61980#issuecomment-503829869
        let slice_box = Box::<[u8]>::from(slice);

        // Calls `FreePointerAndLength`, freeing the allocation created by the foreign code
        drop(foreign_allocation);

        Ok(slice_box)
    }
    /* #endregion */
}
