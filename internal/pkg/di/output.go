package di

import (
	"fmt"
)

var output Output = &StdOutput{}

func GetOutput() Output {
	return output
}

func SetOutput(o Output) {
	output = o
}

type Output interface {
	Print(args... interface{})
	Println(args... interface{})
	Printf(format string, args... interface{})
}

type StdOutput struct {

}

func (o *StdOutput) Print(args... interface{}) {
	fmt.Print(args...)
}

func (o *StdOutput) Println(args... interface{}) {
	fmt.Println(args...)
}

func (o *StdOutput) Printf(format string, args... interface{}) {
	fmt.Printf(format, args...)
}

type TestOutput struct {
	lines []string
}

func (o *TestOutput) GetLines() []string {
	return o.lines
}

func (o *TestOutput) Reset() {
	o.lines = []string{}
}

func (o *TestOutput) Print(args... interface{}) {
	o.lines = append(o.lines, fmt.Sprint(args...))
}

func (o *TestOutput) Println(args... interface{}) {
	o.lines = append(o.lines, fmt.Sprintln(args...))
}

func (o *TestOutput) Printf(format string, args... interface{}) {
	o.lines = append(o.lines, fmt.Sprintf(format, args...))
}