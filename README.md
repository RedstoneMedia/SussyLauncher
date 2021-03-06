
# Sussy Launcher เถ
![Version](https://img.shields.io/github/v/release/RedstoneMedia/SussyLauncher?style=for-the-badge)
![Rocket spam ๐๐๐๐๐ go brrr](https://img.shields.io/badge/blazingfast-%F0%9F%9A%80%F0%9F%92%A8-blueviolet?style=for-the-badge)
![Memory Safe](https://img.shields.io/badge/Memory%20Safe-%F0%9F%94%92-informational?style=for-the-badge)
![Bruh](https://img.shields.io/badge/Kids%20in%20the%20Basement-69%20%F0%9F%98%82%F0%9F%A4%A3-critical?style=for-the-badge)
![GPL-3](https://img.shields.io/github/license/RedstoneMedia/SussyLauncher?style=for-the-badge)
![Issues](https://img.shields.io/github/issues/RedstoneMedia/SussyLauncher?style=for-the-badge)
![Build](https://img.shields.io/github/workflow/status/RedstoneMedia/SussyLauncher/Rust?style=for-the-badge)


A very simple mod launcher/loader for the game Among Us (Also referred to as Aออญอซฬทฬmออฬฌฬอคoฬอขฬฏฬฑอgฬฬฝฬออuอฬฬกฬญฬsฬ าอออ เถ). ๐ฉ๐ฆ

![sussy_launcher_bigger](https://user-images.githubusercontent.com/34373974/140614792-05dbc9a7-0c37-4877-ba57-75b5987ce91b.png)

This Project is written with the Rust programming language ๐ and the tauri framework ๐. \
The use of the rust Programming language is import because of its zero cost abstractions ๐๐โ๐ฑ๐น๐ฐ. \
This means the Launcher is blazing fast ๐๐จ and is Memory safe ๐๐๐พ.

It uses svelte for the frontend ๐ป, which is obviously the best web framework/compiler ๐. \
Apart from using Rust ๐ with WebAssembly ๐๐จ.

Download it [here](https://github.com/RedstoneMedia/SussyLauncher/releases) ๐ฝ \
Either use the installer, or just open the exe (This will create new files at the exe location โผ๐ฒ)

# Contributions
Everyone is welcome to contribute to this. (Why would you want to, though ?) \
I will accept every pull request, that doesn't:
- Completely break everything ๐ฒ๐ฅ
- Contain the word "Go" ๐คก or "Golang" ๐คก anywhere, including the code <- Completely Unacceptable ๐ซ๐โโ๏ธ๐ณ
- Overcomplicated the App. (This Launcher is supposed to be relatively simple and intuitive) ๐ฅถ
- States that anything happened on the 5th April 1976, at Tiananmen Square ๐ทโโ๐ฒใ๐

# Build
If you want to build this yourself, you will need to [install](https://www.rust-lang.org/tools/install) the Rust programming language ๐.
This includes the Rust's ๐ package manager cargo ๐, which is superior to any other package manager ๐ฏ.

Then you need to install the tauri cli like this: ๐ฝ \
`cargo install tauri-cli --version ^1.0.0-beta`

Then you only need to run `npm install` and after that you can already start building. ๐จโ๐ป

For these next steps to work, you might have to [install](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section) WebView2 ๐ถ first.

To run it in dev mode, you can run `cargo tauri dev`. \
If you want to build it, just run `cargo tauri build`. \
The build output then is in `src-tauri/target/release` (Also contains the installers in `/bundle`) ๐

<sup><sub>*(yeah ik I'm trying way too hard to make this "funny")*</sub></sup>
