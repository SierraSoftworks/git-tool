package di

import (
	"io"
	"os"
)

var output Output = os.Stdout

func GetOutput() Output {
	return output
}

func SetOutput(o Output) {
	output = o
}

type Output interface {
	io.Writer
	io.StringWriter
}
