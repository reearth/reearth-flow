# Re:Earth Flow Web

Frontend to build and setup workflows to calculate and convert various types of data through an easy to use UI.

## Development

### Main libraries & services

- React 19
- TypeScript 5
- Vite 7
- Storybook 9
- Yarn 4
- Yjs
- Tanstack Query/Router/Table/Virtual
- Tailwind CSS

### Install prerequisites

Make sure that appropriate environment variables are set, and then run:

```console
yarn

yarn start
// or
yarn storybook
```

For an optimal development experience, make sure you use these vscode settings ([reference]("https://github.com/reearth/eslint-config-reearth/blob/main/.vscode/settings.json")):

```
{
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": "explicit"
  },
  "editor.formatOnSave": true,
  "eslint.enable": true,
  "eslint.useFlatConfig": true,
  "prettier.enable": true,
  "[javascript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[javascriptreact]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[typescriptreact]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

## License

Licensed under either (at your discretion):

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
