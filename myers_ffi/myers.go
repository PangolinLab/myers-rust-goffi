package myers_ffi

/*
	#cgo CFLAGS: -I${SRCDIR}/include
	#cgo LDFLAGS: -lkernel32 -lntdll -luserenv -lws2_32 -ldbghelp -L${SRCDIR}/bin -lmyers
	#include <stdlib.h>
	#include <myers_interface.h>
*/
import "C"

import (
	"unsafe"
)

type EditOp int

const (
	Equal EditOp = iota
	Delete
	Insert
)

type EditRecord struct {
	Op   EditOp
	Line string
}

func init() {
	// 动态库最终路径
	var libFile string
	switch runtime.GOOS {
	case "windows":
		libFile = "bin/myers.dll"
	case "darwin":
		libFile = "bin/libmyers.dylib"
	default:
		libFile = "bin/libmyers.so"
	}

	// 如果库不存在，则编译 Rust 并复制到 bin/
	if _, err := os.Stat(libFile); os.IsNotExist(err) {
		// Rust 源码目录（Cargo.toml 所在目录）
		rustDir := "../" // 根据你的目录结构调整
		buildCmd := exec.Command("cargo", "build", "--release")
		buildCmd.Dir = rustDir
		buildCmd.Stdout = os.Stdout
		buildCmd.Stderr = os.Stderr
		if err := buildCmd.Run(); err != nil {
			panic("Failed to build Rust library: " + err.Error())
		}

		// 源文件路径（默认 target/release/）
		var srcLib string
		switch runtime.GOOS {
		case "windows":
			srcLib = filepath.Join(rustDir, "target", "release", "myers.dll")
		case "darwin":
			srcLib = filepath.Join(rustDir, "target", "release", "libmyers.dylib")
		default:
			srcLib = filepath.Join(rustDir, "target", "release", "libmyers.so")
		}

		// 确保 bin 目录存在
		_ = os.MkdirAll("bin", 0755)

		// 复制库到 bin/
		input, err := os.ReadFile(srcLib)
		if err != nil {
			panic("Failed to read Rust library: " + err.Error())
		}
		if err := os.WriteFile(libFile, input, 0644); err != nil {
			panic("Failed to write library to bin/: " + err.Error())
		}
	}
}

func GetDiffs(oldLines, newLines []string) []EditRecord {
	cOld := make([]*C.char, len(oldLines)+1)
	cNew := make([]*C.char, len(newLines)+1)
	for i, s := range oldLines {
		cOld[i] = C.CString(s)
	}
	cOld[len(oldLines)] = nil
	for i, s := range newLines {
		cNew[i] = C.CString(s)
	}
	cNew[len(newLines)] = nil
	defer func() {
		for _, p := range cOld {
			if p != nil {
				C.free(unsafe.Pointer(p))
			}
		}
		for _, p := range cNew {
			if p != nil {
				C.free(unsafe.Pointer(p))
			}
		}
	}()

	rec := C.diff_lines((**C.char)(&cOld[0]), (**C.char)(&cNew[0]))
	if rec == nil {
		return nil
	}
	defer C.free_diff(rec)

	var goRec []EditRecord
	for p := rec; ; p = (*C.EditRecord)(unsafe.Pointer(uintptr(unsafe.Pointer(p)) + unsafe.Sizeof(*p))) {
		if p.line == nil {
			break
		}
		goRec = append(goRec, EditRecord{
			Op:   EditOp(p.op),
			Line: C.GoString(p.line),
		})
	}
	return goRec
}
