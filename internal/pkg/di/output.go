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
	Print(args ...interface{})
	Println(args ...interface{})
	Printf(format string, args ...interface{})
}

type StdOutput struct {
}

func (o *StdOutput) Write(b []byte) (int, error) {
	return os.Stdout.Write(b)
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

type TestOutput struct {
	operations []string
}

func (o *TestOutput) GetOperations() []string {
	return o.operations
}

func (o *TestOutput) Reset() {
	o.operations = []string{}
}

func (o *TestOutput) Write(b []byte) (int, error) {
	o.operations = append(o.operations, string(b))
	return len(b), nil
}

func (o *TestOutput) Print(args ...interface{}) {
	o.operations = append(o.operations, fmt.Sprint(args...))
}

func (o *TestOutput) Println(args ...interface{}) {
	o.operations = append(o.operations, fmt.Sprintln(args...))
}

func (o *TestOutput) Printf(format string, args ...interface{}) {
	o.operations = append(o.operations, fmt.Sprintf(format, args...))
}
