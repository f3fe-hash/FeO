<!-- Badges --> <p align="center"> <img src="https://img.shields.io/github/stars/f3fe-hash/FeO.svg" /> <img src="https://img.shields.io/github/forks/f3fe-hash/FeO.svg" /> <img src="https://img.shields.io/github/tag/f3fe-hash/FeO.svg" /> <img src="https://img.shields.io/github/release/f3fe-hash/FeO.svg" /> <img src="https://img.shields.io/github/issues/f3fe-hash/FeO.svg" /> <img src="https://img.shields.io/bower/v/FeO.svg" /> </p>
FeO
---

FeO is a Linux-based operating system focused on robotics development. The goal is to provide a practical, flexible platform that can scale from small hobby projects to more complex autonomous systems.

Overview
---

FeO is designed to simplify development for robotics by offering a consistent environment for working with hardware, networking, and system-level components. It targets a wide range of use cases, from basic embedded projects to more advanced robotics applications.


Architecture
---

The system is built using a combination of Rust and C:

* Rust is used where safety and concurrency are important.
* C is used for low-level components and direct hardware interaction.

This approach allows FeO to maintain performance while reducing the risk of common memory-related issues.

Use Cases
---

FeO can be used for:

* Educational robotics projects
* Hobby and DIY robotics
* Embedded systems
* Research and experimentation
* Autonomous robotics systems
* Getting Started

FeO is currently in early development and may be incomplete or unstable.

Documentation
---
FeIO contains a lot of documentation, especially for the Rust part. YOu can access the documentation by going to the `runtime` directory and running this command:
```bash
cargo doc --open
```
This opens the documentation for all of FeO, for example, the c_link module.

To get a local copy:
---
```bash
git clone https://github.com/f3fe-hash/FeO.git
cd FeO

# Compile and run it
make gen_keys
make compile
make run

# In another terminal you can connect to it and send stuff
make client
```

FUP
---
FUP (FeO Utilities Package) is a separate program used to communicate / interface with the FeO machine. It is primarily used for uploading nodes.

Status
---
This project is a work in progress.

Expect breaking changes, incomplete features, and limited documentation.