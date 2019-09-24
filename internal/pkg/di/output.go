package di

import (
	"fmt"
	"io"
	"os"
)

var output Output = &StdOutput{}

func GetOutput() Output {
	return output
}

func SetOutput(o Output) {
	output = o
}

type Output interface {
	io.Writer
	io.StringWriter
	Print(args ...interface{})
	Println(args ...interface{})
	Printf(format string, args ...interface{})
}

type StdOutput struct {
}

func (o *StdOutput) Write(b []byte) (int, error) {
	return os.Stdout.Write(b)
}

func (o *StdOutput) WriteString(s string) (int, error) {
	return os.Stdout.WriteString(s)
}

func (o *StdOutput) Print(args ...interface{}) {
	fmt.Print(args...)
}

func (o *StdOutput) Println(args ...interface{}) {
	fmt.Println(args...)
}

func (o *StdOutput) Printf(format string, args ...interface{}) {
	fmt.Printf(format, args...)
}
