# Sussy Launcher ඞ
A very simple mod launcher/loader for the game Among Us (Also referred to as Amogus ඞ).

![sussy_launcher](https://user-images.githubusercontent.com/34373974/140588628-07d362cb-7bda-44dc-b8ae-1ffc41236e03.png)

This Project is written with the Rust programming language 🚀 and the tauri framework 🚀.
The use of the rust Programming language is import because of its zero cost abstractions 🚀🚀.
This means the Launcher is blazing fast 🚀 and is Memory safe 🚀.

It uses svelte for the frontend.

Download it [here](https://github.com/RedstoneMedia/SussyLauncher/releases)

# Contributions
Everyone is welcome to contribute to this. (Why would you want to though ?)

# Build
If you want to build this yourself, you will need to [install](https://www.rust-lang.org/tools/install) the Rust programming language 🚀.
This includes the Rust's 🚀 package manager cargo 🚀, which is superior to any other package manager.

Then you need to install the tauri cli like this: 
`cargo install tauri-cli --version ^1.0.0-beta`

Then you only need to run `npm install` and after that you can already start building.

For these next steps to work, you might have to [install](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section) WebView2 first.

To run it in dev mode, you can run `cargo tauri dev`. \
If you want to build it, just run `cargo tauri build`.
The build output then is in `src-tauri/target/release` (Also contains the installers in `/bundle`)
