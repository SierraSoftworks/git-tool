package githosts

import (
	"fmt"
	"os"
	"github.com/zalando/go-keyring"
	"golang.org/x/crypto/ssh/terminal"
)

// GetCredentials will retrieve the credentials for a specific Git Tool service
// from the system keychain, if they are present. It will return an error only
// if there is a problem accessing the system keychain.
func GetCredentials(service string) (string, error) {
	creds, err := getCredentials(service)
	if err != nil {
		return "", err
	}

	if creds != "" {
		return creds, nil
	}

	creds, err = readCredentials(service)
	return creds, err
}

func readCredentials(service string) (string, error) {
	fmt.Printf("Enter your access token for %s: ", service) 
	tokenBytes, err := terminal.ReadPassword(int(os.Stdin.Fd()))
	if err != nil {
		return "", err
	}

	token := string(tokenBytes)

	err = keyring.Set("github.com/sierrasoftworks/git-tool", service, token)
	if err != nil {
		return "", err
	}

	return token, nil
}

func getCredentials(service string) (string, error) {
	creds, err := keyring.Get("github.com/sierrasoftworks/git-tool", service)
	if err == nil {
		return creds, nil
	}

	if err == keyring.ErrNotFound {
		return "", nil
	}

	return "", err
}