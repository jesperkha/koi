echo "Installing Koi"

LOCATION="/usr/local"

echo "Creating installation directory"
mkdir koi koi/lib koi/bin
cp README.md koi/
cp LICENSE koi/

echo "Building binary"
cargo build --release -q
cp target/release/koi koi/bin/

echo "Copying build files"
cp lib/entry.s koi/lib/

echo "Moving to /usr/local/"
sudo mv koi /usr/local/

echo "Remember to add /usr/local/koi/bin to PATH!"
echo "Done"