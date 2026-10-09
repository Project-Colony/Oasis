# Changelog

## [0.2.0](https://github.com/Project-Colony/Oasis/compare/v0.1.0...v0.2.0) (2026-10-09)


### Features

* read the config from Colony/Oasis and migrate the old location ([#13](https://github.com/Project-Colony/Oasis/issues/13)) ([2873af6](https://github.com/Project-Colony/Oasis/commit/2873af6521c6524aefab4e68121331081bc745d7))
* ship the binary as oasis with --version and --help ([#9](https://github.com/Project-Colony/Oasis/issues/9)) ([41ad9cf](https://github.com/Project-Colony/Oasis/commit/41ad9cfc7386892630042a3ab616e836a3b27b8f))


### Fixes

* build on macOS and run CI on all three platforms with a read-only token ([#1](https://github.com/Project-Colony/Oasis/issues/1)) ([dd1c7aa](https://github.com/Project-Colony/Oasis/commit/dd1c7aa0e7995c7cfcb1fa3d96f48b860c6a44b5))
* **deps:** update vulnerable crates and replace winrt-notification ([#2](https://github.com/Project-Colony/Oasis/issues/2)) ([70f6414](https://github.com/Project-Colony/Oasis/commit/70f641498bb05fb63ca85599092c9a7e99b42c28))
* let 'quit' stop the daemon and report a malformed config instead of ignoring it ([#8](https://github.com/Project-Colony/Oasis/issues/8)) ([38c5efb](https://github.com/Project-Colony/Oasis/commit/38c5efb790fb55e841427ed758c235c54f8b074a))


### Internals

* move the daemon and the weather lookup out of main.rs ([#11](https://github.com/Project-Colony/Oasis/issues/11)) ([b63c322](https://github.com/Project-Colony/Oasis/commit/b63c322f719be45e780da7f3a378238943e8bf19))
* split monolithic main.rs into modules and apply optimizations ([7a7b4e3](https://github.com/Project-Colony/Oasis/commit/7a7b4e326a5a9323185006a0ca223392db29d704))


### Documentation

* rewrite docs/ in English and list the Linux build prerequisites ([#4](https://github.com/Project-Colony/Oasis/issues/4)) ([16e0c16](https://github.com/Project-Colony/Oasis/commit/16e0c16c9d364818219874ccc09c90e1837ba5a2))
