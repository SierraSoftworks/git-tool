package config

// Features control various configurable behaviors of Git Tool
type Features struct {
	NativeClone  bool `json:"native_clone" yaml:"native_clone"`
	CreateRemote bool `json:"create_remote" yaml:"create_remote"`
}

func defaultFeatures() *Features {
	return &Features{
		NativeClone:  false,
		CreateRemote: true,
	}
}

func (f *Features) UseNativeClone() bool {
	return f.NativeClone
}

func (f *Features) CreateRemoteRepo() bool {
	return f.CreateRemote
}
