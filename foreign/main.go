// Referenced https://blog.arranfrance.com/post/cgo-sqip-rust/ for Rust -> Go FFI information
package main

// #include <stdlib.h>
//
// enum {
// 	FAILURE_CODE = 2,
// 	SUCCESS_CODE = 1
// };
//
// struct pointer_and_length {
// 	void* a_pointer;
// 	int b_length;
// };
//
// struct decompress_bzip_two_return_type {
// 	unsigned char a_status_code;
// 	struct pointer_and_length b_error_message;
// 	struct pointer_and_length c_data;
// };
//
// struct convert_rar_to_tar_return_type {
// 	unsigned char a_status_code;
// 	struct pointer_and_length b_error_message;
// 	struct pointer_and_length c_data;
// };
import (
	"C"
)

import (
	"archive/tar"
	"bytes"
	"compress/bzip2"
	"errors"
	"fmt"
	"io"

	"github.com/nwaples/rardecode"
)

type ConvertRarToTarReturnType = C.struct_convert_rar_to_tar_return_type
type DecompressBzipTwoReturnType = C.struct_decompress_bzip_two_return_type
type PointerAndLength = C.struct_pointer_and_length

const failureCode C.uchar = C.FAILURE_CODE
const successCode C.uchar = C.SUCCESS_CODE

const unexpectedNilEncounteredErrorMessage string = "Unexpected nil encountered"

func main() {}

//export ConvertRarToTar
func ConvertRarToTar(dataPointerAndLength PointerAndLength, passwordPointerAndLength PointerAndLength) ConvertRarToTarReturnType {
	dataUintEightArray := PointerAndLengthToUintEightArray(dataPointerAndLength)
	passwordUintEightArray := PointerAndLengthToUintEightArray(passwordPointerAndLength)

	ui, er := ConvertRarToTarInner(dataUintEightArray, passwordUintEightArray)

	if er != nil {
		erString := fmt.Sprint(er)

		return ConvertRarToTarReturnType{
			a_status_code:   failureCode,
			b_error_message: StringToToPointerAndLength(erString),
			c_data:          EmptyUintEightArrayToPointerAndLength(),
		}
	}

	if ui == nil {
		return ConvertRarToTarReturnType{
			a_status_code:   failureCode,
			b_error_message: StringToToPointerAndLength(unexpectedNilEncounteredErrorMessage),
			c_data:          EmptyUintEightArrayToPointerAndLength(),
		}
	} else {
		return ConvertRarToTarReturnType{
			a_status_code:   successCode,
			b_error_message: EmptyUintEightArrayToPointerAndLength(),
			c_data:          UintEightArrayToPointerAndLength(ui),
		}
	}
}

// Referenced https://medium.com/@s.vvardenfell/creating-in-memory-tar-archive-in-go-golang-83b7ca309602
func ConvertRarToTarInner(dataUintEightArray []uint8, passwordUintEightArray []uint8) ([]uint8, error) {
	// TODO
	// How much memory should be preallocated?
	in := len(dataUintEightArray) * 4

	bu := bytes.NewBuffer(make([]uint8, 0, in))

	{
		st := string(passwordUintEightArray)

		re, er := rardecode.NewReader(bytes.NewBuffer(dataUintEightArray), st)

		if er != nil {
			return nil, er
		}

		{
			wr := tar.NewWriter(bu)

			for {
				fi, err := re.Next()

				if err != nil {
					if err != io.EOF {
						return nil, err
					}

					break
				}

				unPackedSize := fi.UnPackedSize

				// TODO
				// Is this the right amount of memory to preallocate?
				buf := bytes.NewBuffer(make([]uint8, 0, unPackedSize))

				intS, erro := io.Copy(buf, re)

				if erro != nil {
					return nil, erro
				}

				name := fi.Name

				if intS != unPackedSize {
					fmt.Printf("WARNING: Mismatch between number of actually read bytes (%d) and size reported in header (%d) when processing file \"%s\"", intS, unPackedSize, name)
				}

				fil := fi.Mode()

				var typeflag uint8

				if fil.IsDir() {
					typeflag = tar.TypeDir
				} else if fil.IsRegular() {
					typeflag = tar.TypeReg
				} else {
					return nil, errors.New("unexpected file type encountered")
				}

				he := tar.Header{
					Mode:     int64(fil),
					ModTime:  fi.ModificationTime,
					Name:     name,
					Size:     intS,
					Typeflag: typeflag,
				}

				errorR := wr.WriteHeader(&he)

				if errorR != nil {
					return nil, errorR
				}

				_, errorRr := io.Copy(wr, buf)

				if errorRr != nil {
					return nil, errorRr
				}
			}

			wr.Close()
		}
	}

	ui := bu.Bytes()

	return ui, nil
}

//export DecompressBzipTwo
func DecompressBzipTwo(dataPointerAndLength PointerAndLength) DecompressBzipTwoReturnType {
	dataUintEightArray := PointerAndLengthToUintEightArray(dataPointerAndLength)

	ui, er := DecompressBzipTwoInner(dataUintEightArray)

	if er != nil {
		erString := fmt.Sprint(er)

		return DecompressBzipTwoReturnType{
			a_status_code:   failureCode,
			b_error_message: StringToToPointerAndLength(erString),
			c_data:          EmptyUintEightArrayToPointerAndLength(),
		}
	}

	if ui == nil {
		return DecompressBzipTwoReturnType{
			a_status_code:   failureCode,
			b_error_message: StringToToPointerAndLength(unexpectedNilEncounteredErrorMessage),
			c_data:          EmptyUintEightArrayToPointerAndLength(),
		}
	} else {
		return DecompressBzipTwoReturnType{
			a_status_code:   successCode,
			b_error_message: EmptyUintEightArrayToPointerAndLength(),
			c_data:          UintEightArrayToPointerAndLength(ui),
		}
	}
}

func DecompressBzipTwoInner(dataUintEightArray []uint8) ([]uint8, error) {
	// TODO
	// How much memory should be preallocated?
	in := len(dataUintEightArray) * 4

	bu := bytes.NewBuffer(make([]uint8, 0, in))

	{
		re := bzip2.NewReader(bytes.NewBuffer(dataUintEightArray))

		_, er := io.Copy(bu, re)

		if er != nil {
			return nil, er
		}
	}

	ui := bu.Bytes()

	return ui, nil
}

//export FreePointerAndLength
func FreePointerAndLength(targetPointerAndLength PointerAndLength) {
	C.free(targetPointerAndLength.a_pointer)
}

func EmptyUintEightArrayToPointerAndLength() PointerAndLength {
	return UintEightArrayToPointerAndLength([]uint8{})
}

func PointerAndLengthToUintEightArray(po PointerAndLength) []uint8 {
	return C.GoBytes(po.a_pointer, po.b_length)
}

func StringToToPointerAndLength(st string) PointerAndLength {
	return UintEightArrayToPointerAndLength([]uint8(st))
}

func UintEightArrayToPointerAndLength(ui []uint8) PointerAndLength {
	return PointerAndLength{
		a_pointer: C.CBytes(ui),
		b_length:  C.int(len(ui)),
	}
}
