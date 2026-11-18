# Documentation

Documentação em português: veja a coluna da direita.

## Guides for everyone

| Guide | English | Português |
|---|---|---|
| Install the app and open it | [Getting started](guide/getting-started.md) | [Primeiros passos](guide/getting-started.pt-BR.md) |
| Every screen, step by step, and the command line | [Usage](guide/usage.md) | [Uso](guide/usage.pt-BR.md) |
| What can and cannot be deleted, and what is not tested yet | [Safety](guide/safety.md) | [Segurança](guide/safety.pt-BR.md) |
| Common questions and error messages | [Questions and troubleshooting](guide/faq.md) | [Perguntas e solução de problemas](guide/faq.pt-BR.md) |

## For contributors

These are written in English.

| Document | What it covers |
|---|---|
| [Architecture](architecture.md) | The layers, the dependency rule, the three binaries, how a feature and a removal are laid out |
| [Conventions](conventions.md) | Naming, no duplicated code, code quality, the quality gate |
| [Security policy](security.md) | The rules the code must follow to delete safely, and how the privileged helper works |
| [Decision records](adr/) | Why the main decisions were made |
| [Contributing](../CONTRIBUTING.md) | How to propose a change, the project tasks, how to add a cleanup rule |

## Design

The interface follows a design reference: [`design/prototype/index.html`](../design/prototype/index.html). Open it in a browser. Each screen has its own address, such as `index.html#cleanup` or `index.html#disk-map`. It is a reference for how screens look; what the app does is described in the guides above.

## Images

The images in this documentation are in [`assets/`](assets/). They are screenshots of the app itself, run on demonstration data, so no personal file appears in them. [CONTRIBUTING.md](../CONTRIBUTING.md#updating-the-screenshots) explains how to produce them again.
