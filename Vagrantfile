# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "ubuntu/focal64"

  # Start shell in our package directory.
  config.ssh.extra_args = ["-t", "cd /vagrant; ls; bash --login"]

  config.vm.network "forwarded_port", guest: 5000, host: 8080
    
  config.vm.provider "virtualbox" do |vb|
    # Customize the amount of memory on the VM:
    vb.memory = "4096"
    vb.cpus = 2
  end

  config.vm.provision "file", source: "./vendor/rustup-init.sh", destination: "~/rustup-init.sh"
  config.vm.provision "shell", privileged: false, inline: <<-SHELL
    sudo apt-get install -y build-essential pkg-config libssl-dev # Rust dependencies
    sh $HOME/rustup-init.sh -y
    source $HOME/.cargo/env

    sudo snap install go --classic
    sudo snap install ruby --classic
    sudo apt-get update
    sudo apt-get install -y gdb
    pip install gdbgui
  SHELL
end

