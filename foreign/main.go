// Referenced https://blog.arranfrance.com/post/cgo-sqip-rust/ for Rust -> Go FFI information
package main

// #include <stdlib.h>
// #define FAILURE_CODE_DEFINE 2
// #define SUCCESS_CODE_DEFINE 1
// enum {
// 	FAILURE_CODE = FAILURE_CODE_DEFINE,
// 	SUCCESS_CODE = SUCCESS_CODE_DEFINE
// };
// typedef struct pointer_and_length {
// 	void* a_pointer;
// 	int b_length;
// } pointer_and_length;
// typedef struct decompress_bzip_two_return_type {
// 	unsigned char a_status_code;
// 	pointer_and_length b_error_message;
// 	pointer_and_length c_data;
// } decompress_bzip_two_return_type;
// typedef struct convert_rar_to_tar_return_type {
// 	unsigned char a_status_code;
// 	pointer_and_length b_error_message;
// 	pointer_and_length c_data;
// } convert_rar_to_tar_return_type;
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

const failureCode C.uchar = C.FAILURE_CODE_DEFINE
const successCode C.uchar = C.SUCCESS_CODE_DEFINE

func main() {}

//export ConvertRarToTar
func ConvertRarToTar(dataPointerAndLength C.struct_pointer_and_length, passwordPointerAndLength C.struct_pointer_and_length) C.struct_convert_rar_to_tar_return_type {
	dataByteArray := PointerAndLengthToByteArray(dataPointerAndLength)
	passwordByteArray := PointerAndLengthToByteArray(passwordPointerAndLength)

	by, er := ConvertRarToTarInner(dataByteArray, passwordByteArray)

	if er != nil {
		erString := fmt.Sprint(er)

		return C.struct_convert_rar_to_tar_return_type{
			a_status_code:   failureCode,
			b_error_message: StringToToPointerAndLength(erString),
			c_data:          EmptyByteArrayToPointerAndLength(),
		}
	}

	return C.struct_convert_rar_to_tar_return_type{
		a_status_code:   successCode,
		b_error_message: EmptyByteArrayToPointerAndLength(),
		c_data:          ByteArrayToPointerAndLength(by),
	}
}

// Referenced https://medium.com/@s.vvardenfell/creating-in-memory-tar-archive-in-go-golang-83b7ca309602
func ConvertRarToTarInner(dataByteArray []byte, passwordByteArray []byte) ([]byte, error) {
	bu := bytes.NewBuffer(dataByteArray)
	st := string(passwordByteArray)

	re, er := rardecode.NewReader(bu, st)

	if er != nil {
		return nil, er
	}

	buf := bytes.Buffer{}

	wr := io.Writer(&buf)

	wri := tar.NewWriter(wr)

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
		byteA := make([]byte, 0, unPackedSize)

		buff := bytes.NewBuffer(byteA)

		in, erro := io.Copy(buff, re)

		if erro != nil {
			return nil, erro
		}

		name := fi.Name

		if in != unPackedSize {
			fmt.Printf("WARNING: Mismatch between number of actually read bytes (%d) and size reported in header (%d) when processing file \"%s\"", in, unPackedSize, name)
		}

		mo := fi.Mode()

		var typeflag byte

		if mo.IsDir() {
			typeflag = tar.TypeDir
		} else if mo.IsRegular() {
			typeflag = tar.TypeReg
		} else {
			return nil, errors.New("unexpected file type encountered")
		}

		he := tar.Header{
			Mode:     int64(mo),
			ModTime:  fi.ModificationTime,
			Name:     name,
			Size:     in,
			Typeflag: typeflag,
		}

		errorR := wri.WriteHeader(&he)

		if errorR != nil {
			return nil, errorR
		}

		_, errorRr := io.Copy(wri, buff)

		if errorRr != nil {
			return nil, errorRr
		}
	}

	wri.Close()

	byteAr := buf.Bytes()

	return byteAr, nil
}

//export DecompressBzipTwo
func DecompressBzipTwo(dataPointerAndLength C.struct_pointer_and_length) C.struct_decompress_bzip_two_return_type {
	dataByteArray := PointerAndLengthToByteArray(dataPointerAndLength)

	by, er := DecompressBzipTwoInner(dataByteArray)

	if er != nil {
		erString := fmt.Sprint(er)

		return C.struct_decompress_bzip_two_return_type{
			a_status_code:   failureCode,
			b_error_message: StringToToPointerAndLength(erString),
			c_data:          EmptyByteArrayToPointerAndLength(),
		}
	}

	return C.struct_decompress_bzip_two_return_type{
		a_status_code:   successCode,
		b_error_message: EmptyByteArrayToPointerAndLength(),
		c_data:          ByteArrayToPointerAndLength(by),
	}
}

func DecompressBzipTwoInner(dataByteArray []byte) ([]byte, error) {
	bu := bytes.NewBuffer(dataByteArray)

	re := bzip2.NewReader(bu)

	// TODO
	buf := bytes.NewBuffer(make([]byte, 0, len(dataByteArray)*10))

	_, er := io.Copy(buf, re)

	if er != nil {
		return nil, er
	}

	byt := buf.Bytes()

	return byt, nil
}

//export FreePointerAndLength
func FreePointerAndLength(pointerAndLength C.struct_pointer_and_length) {
	C.free(pointerAndLength.a_pointer)
}

func ByteArrayToPointerAndLength(by []byte) C.struct_pointer_and_length {
	return C.struct_pointer_and_length{
		a_pointer: C.CBytes(by),
		b_length:  C.int(len(by)),
	}
}

func EmptyByteArrayToPointerAndLength() C.struct_pointer_and_length {
	by := []byte{}

	return ByteArrayToPointerAndLength(by)
}

func PointerAndLengthToByteArray(pointerAndLength C.struct_pointer_and_length) []byte {
	return C.GoBytes(pointerAndLength.a_pointer, pointerAndLength.b_length)
}

func StringToToPointerAndLength(st string) C.struct_pointer_and_length {
	by := []byte(st)

	return ByteArrayToPointerAndLength(by)
}
