package mocks

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
