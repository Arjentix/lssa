set -e
curl -L https://risczero.com/install | bash 
/Users/.risc0/bin/rzup install 
cargo build