# ⚠️ Still in development ⚠️

# teabox - A simple file hosting server in rust

Accepts file and saved them in files named as the sha256 digest of content.
Similar to [0x0.st](https://0x0.st/)

## Usage

### Post a file

```
curl -F 'file=@yourfile' 127.0.0.1:7878
```
