build:
  cargo build --release -p nudo
  sudo chown -R root:root target/release/nudo
  sudo chmod 4775 target/release/nudo
install:
  sudo rm -rf /usr/local/bin/nudo
  cargo build --release nudo
  sudo cp target/release/nudo /usr/local/bin/
build-debug:
  cargo build -p nudo
  sudo chown -R root:root target/debug/nudo
  sudo chmod 4775 target/debug/nudo
install-debug:
  sudo rm -rf /usr/local/bin/nudo
  cargo build nudo
  sudo cp target/debug/nudo /usr/local/bin/
clean:
  cargo clean
uninstall:
  sudo rm -rf /usr/local/bin/nudo*
