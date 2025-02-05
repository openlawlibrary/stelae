1. Install Docker
2. From powershell run `cd <path to stelae project root>` and then run `docker build -t rust-compiler .` to download image
3. After image is downloaded run `docker run --rm -v "C:\<pathtostelae>\:/workspace" rust-compiler bash -c "cargo build --release"`