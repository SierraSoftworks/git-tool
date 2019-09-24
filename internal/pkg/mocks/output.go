package mocks

import (
	"fmt"
)

type Output struct {
	operations []string
}

func (o *Output) GetOperations() []string {
	return o.operations
}

func (o *Output) Reset() {
	o.operations = []string{}
}

func (o *Output) Write(b []byte) (int, error) {
	o.operations = append(o.operations, string(b))
	return len(b), nil
}

func (o *Output) WriteString(s string) (int, error) {
	o.operations = append(o.operations, s)
	return len(s), nil
}

func (o *Output) Print(args ...interface{}) {
	o.operations = append(o.operations, fmt.Sprint(args...))
}

func (o *Output) Println(args ...interface{}) {
	o.operations = append(o.operations, fmt.Sprintln(args...))
}

func (o *Output) Printf(format string, args ...interface{}) {
	o.operations = append(o.operations, fmt.Sprintf(format, args...))
}
