# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [v0.10.0](https://github.com/rtuszik/pls-reply/compare/046d9061a0746606ce3d2c08f82ba82e754eb3af..v0.10.0) - 2026-10-11
#### Features
- (**config**) add ordered fallback models for failed requests - ([03a95e5](https://github.com/rtuszik/pls-reply/commit/03a95e534af90266793c28b7f9821bc752de48e5)) - Robin Tuszik
- (**llm**) trim leading whitespace before first content - ([a4c07c5](https://github.com/rtuszik/pls-reply/commit/a4c07c597abd371c678c2117ef214ae1458b694c)) - Robin Tuszik
#### Bug Fixes
- (**mise**) remove hyperfine linux-arm64-musl lock entry - ([9893662](https://github.com/rtuszik/pls-reply/commit/9893662c9e2941285b3d4b1d60523b949a0a5ef1)) - Robin Tuszik
#### Continuous Integration
- add dispatchable baseline benchmark workflow - ([6d6c4a4](https://github.com/rtuszik/pls-reply/commit/6d6c4a4eee70aaae24bbdb6017a01213ecb23c0a)) - Robin Tuszik
#### Refactoring
- (**llm**) write to provided writer and add fallback tests - ([eeb6bd1](https://github.com/rtuszik/pls-reply/commit/eeb6bd15f8ff65e0fa2a4593aae2dea9ff5a2b98)) - Robin Tuszik
#### Miscellaneous Chores
- (**deps**) update pre-commit hook betterleaks/betterleaks to v1.9.0 - ([59ee101](https://github.com/rtuszik/pls-reply/commit/59ee10134e8dadbb129bdb49a4fd57ea0a7f4007)) - koalabot-rt[bot]
- (**deps**) update dependency prek to latest - ([20d3929](https://github.com/rtuszik/pls-reply/commit/20d3929164f522a4caf8ce16a53b3092eb0391dc)) - koalabot-rt[bot]
- (**deps**) update dependency vhs to latest - ([277316f](https://github.com/rtuszik/pls-reply/commit/277316f879013d2314217a8f116d99371ee0114e)) - koalabot-rt[bot]
- (**deps**) update dependency rust to latest - ([99a85c5](https://github.com/rtuszik/pls-reply/commit/99a85c578b9bf5769c0fb5275ef2d42a585fcd5e)) - koalabot-rt[bot]
- (**deps**) update jdx/mise-action action to v5 - ([81eb46d](https://github.com/rtuszik/pls-reply/commit/81eb46d8208053d00d413fc085a102224650defb)) - koalabot-rt[bot]
- (**deps**) update dependency tombi to latest - ([25c8942](https://github.com/rtuszik/pls-reply/commit/25c89421144ca18d7de1c491145549565fe3f441)) - koalabot-rt[bot]
- (**deps**) update dependency usage to v6.12.1 - ([9c696fb](https://github.com/rtuszik/pls-reply/commit/9c696fb8ba8340ab233eec800c8660002ef07ace)) - koalabot-rt[bot]
- (**deps**) update dependency opengrep to latest - ([da7348b](https://github.com/rtuszik/pls-reply/commit/da7348b08cdcf438079960fcfa535fc950345897)) - koalabot-rt[bot]
- (**deps**) update dependency hyperfine to v2 - ([93ad17a](https://github.com/rtuszik/pls-reply/commit/93ad17ada3a8e7782aa952d20d41442a94f3671b)) - koalabot-rt[bot]
- (**deps**) update dependency opengrep to latest (#35) - ([064a70e](https://github.com/rtuszik/pls-reply/commit/064a70e283d29cebe84cd6ca5dc0c0a3efcfba7e)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) update rust crate tokio to v1.53.2 (#34) - ([e8c8dc3](https://github.com/rtuszik/pls-reply/commit/e8c8dc3f26f848f82f136521727d0001ec79bc31)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) update dependency tombi to latest (#26) - ([046d906](https://github.com/rtuszik/pls-reply/commit/046d9061a0746606ce3d2c08f82ba82e754eb3af)) - koalabot-rt[bot], koalabot-rt[bot]
- (**mise**) update lockfile - ([0c6b511](https://github.com/rtuszik/pls-reply/commit/0c6b511257324238857aca88433b0567bdf893a2)) - Robin Tuszik
- (**pre-commit**) add default stages - ([196b6d0](https://github.com/rtuszik/pls-reply/commit/196b6d0388eaca3a625efe484cb52ad4a53e3af0)) - Robin Tuszik

- - -

## [v0.9.0](https://github.com/rtuszik/pls-reply/compare/97976b3b4afb57070f11b93229dc4545d9820bb4..v0.9.0) - 2026-09-20
#### Features
- ![BREAKING](https://img.shields.io/badge/BREAKING-red) add commit subcommand and use ask/commit commands - ([97976b3](https://github.com/rtuszik/pls-reply/commit/97976b3b4afb57070f11b93229dc4545d9820bb4)) - Robin Tuszik
#### Miscellaneous Chores
- simplify commit message prompt instructions - ([a8f85cf](https://github.com/rtuszik/pls-reply/commit/a8f85cf14e79733ccc740950d6f6b9c4746729bb)) - Robin Tuszik
- add benchmark query asset - ([29e5426](https://github.com/rtuszik/pls-reply/commit/29e54261c672c56ab1c708ab54899d84bb2881e5)) - Robin Tuszik

- - -

## [v0.8.0](https://github.com/rtuszik/pls-reply/compare/018f8bad2bb96ee6001652a0dc3a64c6e7b671da..v0.8.0) - 2026-09-16
#### Features
- add usage tool configuration - ([4888dcf](https://github.com/rtuszik/pls-reply/commit/4888dcf7b13fc73915655c9f97c68db3b9447adc)) - Robin Tuszik
#### Refactoring
- (**cli**) replace clap with usage-rs argument parsing - ([018f8ba](https://github.com/rtuszik/pls-reply/commit/018f8bad2bb96ee6001652a0dc3a64c6e7b671da)) - Robin Tuszik

- - -

## [v0.7.1](https://github.com/rtuszik/pls-reply/compare/9c3782b336363fdac07982936c71a68ead53797b..v0.7.1) - 2026-09-16
#### Bug Fixes
- (**stats**) report effective throughput over request duration - ([9c3782b](https://github.com/rtuszik/pls-reply/commit/9c3782b336363fdac07982936c71a68ead53797b)) - Robin Tuszik

- - -

## [v0.7.0](https://github.com/rtuszik/pls-reply/compare/54bbd717474e0b6ffec4a3daaeb9975f8b5b6cae..v0.7.0) - 2026-09-14
#### Features
- (**cli**) add latency profiling with text and JSON output - ([54bbd71](https://github.com/rtuszik/pls-reply/commit/54bbd717474e0b6ffec4a3daaeb9975f8b5b6cae)) - Robin Tuszik

- - -

## [v0.6.0](https://github.com/rtuszik/pls-reply/compare/9f2910c10ce3844ab0201b84bc45d87e3ae89142..v0.6.0) - 2026-09-14
#### Features
- (**cli**) add JSON statistics output for model requests - ([d10c0d3](https://github.com/rtuszik/pls-reply/commit/d10c0d3a3a6c237553a15c286812c9b692825bc6)) - Robin Tuszik
#### Continuous Integration
- run tests through mise task and update tools - ([43079bd](https://github.com/rtuszik/pls-reply/commit/43079bda106488c2e8c18ff52bc2642a7d840c3b)) - Robin Tuszik
#### Miscellaneous Chores
- (**deps**) update rust crate toml to v1.1.6 (#19) - ([9596eb6](https://github.com/rtuszik/pls-reply/commit/9596eb6f383fbb8cd936bba6bbcc70c6290e7d48)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) update rust crate toml to v1.1.5 (#17) - ([9f2910c](https://github.com/rtuszik/pls-reply/commit/9f2910c10ce3844ab0201b84bc45d87e3ae89142)) - koalabot-rt[bot], koalabot-rt[bot]

- - -

## [v0.5.0](https://github.com/rtuszik/pls-reply/compare/9e1c5504ce35f5a7dfb7efa857375b0e65b097b9..v0.5.0) - 2026-09-01
#### Features
- support dynamic provider adapter resolution - ([8f82f9e](https://github.com/rtuszik/pls-reply/commit/8f82f9ee35007dea939f813f83a9a5c809ad2430)) - Robin Tuszik
#### Bug Fixes
- (**config**) preserve schema directive in default config template - ([522d051](https://github.com/rtuszik/pls-reply/commit/522d0517677c84b8e377d507d2705243d198289a)) - Robin Tuszik
#### Documentation
- streamline README installation and usage guidance - ([b88cc99](https://github.com/rtuszik/pls-reply/commit/b88cc996afa0c713965c9997eb6592ada667c683)) - Robin Tuszik
#### Build system
- set minimum supported Rust version to 1.85 - ([9e1c550](https://github.com/rtuszik/pls-reply/commit/9e1c5504ce35f5a7dfb7efa857375b0e65b097b9)) - Robin Tuszik

- - -

## [v0.4.0](https://github.com/rtuszik/pls-reply/compare/6313722e4101e99e06a69e5edc21d8989c635e2c..v0.4.0) - 2026-08-31
#### Features
- (**config**) add TOML schema and linting support - ([9102192](https://github.com/rtuszik/pls-reply/commit/9102192bd722053e43edb7306228e4da3885cfe8)) - Robin Tuszik
#### Continuous Integration
- filter release tag output in workflow - ([faa13c7](https://github.com/rtuszik/pls-reply/commit/faa13c7c2195fe2ccd9ff5e08a9d824197b55c6e)) - Robin Tuszik
#### Miscellaneous Chores
- (**deps**) update jdx/mise-action action to v4.3.0 - ([bdd2c36](https://github.com/rtuszik/pls-reply/commit/bdd2c36cf6b23821dab762519da66c9c37993c2b)) - koalabot-rt[bot]
- (**deps**) update actions/checkout action to v7 - ([6313722](https://github.com/rtuszik/pls-reply/commit/6313722e4101e99e06a69e5edc21d8989c635e2c)) - koalabot-rt[bot]

- - -

## [v0.3.0](https://github.com/rtuszik/pls-reply/compare/900267b79cfe18c86bd607962eeddc36c971c519..v0.3.0) - 2026-08-30
#### Features
- add demo recording and update installation docs - ([5d81207](https://github.com/rtuszik/pls-reply/commit/5d812076a847ac11d9baa832a5245e9b2c75fb1e)) - Robin Tuszik
#### Bug Fixes
- (**deps**) update rust crate toml to v1 - ([1721bc6](https://github.com/rtuszik/pls-reply/commit/1721bc64db840139034f62989171551a83b1a5d1)) - koalabot-rt[bot]
#### Build system
- add betterleaks pre-commit hook - ([8ff6632](https://github.com/rtuszik/pls-reply/commit/8ff66329da16fd1590a3173238dcf4f3d82155e0)) - Robin Tuszik
#### Continuous Integration
- add automated CI and release workflows - ([998a5d8](https://github.com/rtuszik/pls-reply/commit/998a5d8db6e64d1bd319791ef1655466eec482c5)) - Robin Tuszik
#### Miscellaneous Chores
- (**deps**) update rust crate futures to v0.3.34 - ([c18d99d](https://github.com/rtuszik/pls-reply/commit/c18d99d9ee7cf674dbd3a1d12922ae876bf1a003)) - koalabot-rt[bot]
- (**deps**) update jdx/mise-action action to v4.2.5 (#13) - ([197ba85](https://github.com/rtuszik/pls-reply/commit/197ba85cc2f720017dcf22d2f5c60797c3c95b68)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) update rust crate clap to v4.6.6 (#11) - ([17e5c52](https://github.com/rtuszik/pls-reply/commit/17e5c52db99e7c3d7acd9e5a141e1b3c2bf7ed04)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) update swatinem/rust-cache action to v2.9.2 (#10) - ([ec5981f](https://github.com/rtuszik/pls-reply/commit/ec5981f0cafe45c655b5ebbf3d1d5e90ad3e1b7d)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) update jdx/mise-action action to v4.2.4 (#9) - ([31b3b26](https://github.com/rtuszik/pls-reply/commit/31b3b2600f755089424c7cc503a24ab561c6dcb7)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) update rust crate clap to v4.6.5 (#8) - ([bcd7211](https://github.com/rtuszik/pls-reply/commit/bcd7211ea543aff59fe7c5f4ecd7498abd9e3c6f)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) update rust crate toml to v1.1.4 (#7) - ([35bec44](https://github.com/rtuszik/pls-reply/commit/35bec441c2b961abb9a8c46206c3c86b4747cbc4)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) update jdx/mise-action action to v4.2.3 (#6) - ([067f2c6](https://github.com/rtuszik/pls-reply/commit/067f2c6446cc92fa359f271428075b75c2b9c5e1)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) update jdx/mise-action action to v4.2.2 (#5) - ([c497db4](https://github.com/rtuszik/pls-reply/commit/c497db4e19ae403384bfcff637d351f2bd03926b)) - koalabot-rt[bot], koalabot-rt[bot]
- (**deps**) add renovate.json - ([900267b](https://github.com/rtuszik/pls-reply/commit/900267b79cfe18c86bd607962eeddc36c971c519)) - koalabot-rt[bot]
#### Style
- reformat Renovate configuration indenting - ([5f9db98](https://github.com/rtuszik/pls-reply/commit/5f9db98ccdaa6bb03e29c30cd01bd3b0a4900bc6)) - Robin Tuszik

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).