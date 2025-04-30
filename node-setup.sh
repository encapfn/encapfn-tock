#!/bin/bash

# Check whether Nix is installed, otherwise prompt the user to install it. It
# might be installed, but the current shell session may not have it included
# in its path. Thus we check for the existence of `/nix` instead.

if [ ! -d /nix ]; then
    echo "Nix does not seem to be installed on this node, press ENTER to install it"
    read
    sh <(curl -L https://nixos.org/nix/install) --daemon --yes
fi

# Ensure that the Nix tools are accessible to this script:
. '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh'

# Make sure we're running in a Nix shell with all of our required dependencies,
# and are running as root. Otherwise re-launch this script through sudo and
# nix-shell:
if [ "$IN_RELAUNCHED_NIX_SHELL" != "yes" ]; then
    echo "Relaunching script as root within nix-shell environment..."
    exec sudo HOME=/root "$(which nix-shell)" \
        -p linuxPackages.cpupower util-linux \
	--run "IN_RELAUNCHED_NIX_SHELL=yes bash \"$(readlink -f "$0")\" \"$@\""
fi

function banner() {
    RED='\033[0;31m'
    NC='\033[0m' # No Color
    printf "\n\n${RED}========== %s ========== ${NC}\n" "$1"
}

# Print useful information for reproducibility:
banner "Printing node & CPU information"
hostname
lscpu

# Install required packages:
banner "Installing required dependencies"
sudo DEBIAN_FRONTEND=noninteractive apt-get update
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y pkg-config libudev-dev libssl-dev hostname libncurses5 libftdi1-dev build-essential picocom wget

# Generate locales (required for Xilinx tools)
banner "Generate en_US.UTF-8 locales"
echo "LC_ALL=en_US.UTF-8" | sudo tee -a /etc/environment
echo "en_US.UTF-8 UTF-8" | sudo tee -a /etc/locale.gen
echo "LANG=en_US.UTF-8" | sudo tee /etc/locale.conf
sudo locale-gen en_US.UTF-8

