# Uso

[English](usage.md) · Português

Como usar cada tela do MacCleaner e a ferramenta de linha de comando. Se ainda não instalou, comece pelos [Primeiros passos](getting-started.pt-BR.md).

> As imagens desta página são capturas do app com dados de demonstração. As suas pastas, tamanhos e programas serão outros.

## A janela

![A janela do MacCleaner na tela do painel](../assets/screens/dashboard.png)

| Área | O que mostra |
|---|---|
| **Barra de status** (em cima) | O nome do app, o espaço livre no disco de inicialização, a hora e o ícone de aparência. |
| **Trilho de navegação** (à esquerda) | As sete telas, cada uma com a sua tecla numérica. A que está acesa é onde você está. |
| **Medidor do disco** (embaixo, à esquerda) | "Disco de inicialização": quão cheio ele está. Fica âmbar e mostra "crítico" quando há menos de 10% livre. |
| **Área de trabalho** (à direita) | O título e uma descrição de uma linha da tela atual, e depois o conteúdo. |

### Teclado

| Tecla | Ação |
|---|---|
| `1` a `7` | Ir para uma tela, na ordem do trilho |
| `Esc` | Fechar o painel de aparência |
| `Command + Q` | Sair |

### Aparência

Clique no ícone de controles, na ponta direita da barra de status, para abrir o painel "Aparência".

| Controle | Opções |
|---|---|
| "Paleta" | "Agência" (ciano), "Terminal" (verde), "Tático" (âmbar) |
| "Animações" | "Completa": tudo se move. "Reduzida": só transições curtas, nada fica em repetição. "Desligada": nenhuma animação. |
| "Brilho" | A intensidade do brilho em volta de títulos e destaques. Todo para a esquerda desliga. |
| "Rever abertura" | Mostra de novo a sequência de abertura. |

![O painel nas três paletas: Agência, Terminal e Tático](../assets/palettes.png)

Feche o painel com `Esc` ou clicando fora dele.

Vale saber:

- Essas escolhas não são salvas: o app abre com os padrões a cada vez.
- Se "Reduzir movimento" estiver ligado na parte de Acessibilidade dos Ajustes do Sistema, o app abre com "Animações" em "Reduzida" e pula a sequência de abertura. Você ainda pode escolher outro nível no painel. Esse comportamento na abertura não foi testado com a opção ligada.
- Para poupar bateria, a janela é redesenhada só uma vez por segundo sempre que não está na frente.
- Com "Completa", o app usa mais do processador enquanto a janela está na frente (de 10 a 15% de um núcleo em um Apple M2 Pro). "Reduzida" diminui esse uso.

### O aviso de Acesso Total ao Disco

O macOS esconde algumas pastas de todo app até você permitir. Quando essa permissão falta, as telas que leem as suas pastas ("Limpeza", "Duplicados", "Fotos", "Desenvolvedor" e "Mapa do disco") mostram uma linha no topo:

> Sem Acesso Total ao Disco, o macOS esconde parte da sua pasta pessoal das análises. **Abrir ajustes de Acesso Total ao Disco**

Clique no link "Abrir ajustes de Acesso Total ao Disco" para ir à página certa dos Ajustes do Sistema. O [guia de primeiros passos](getting-started.pt-BR.md#passo-7-dê-o-acesso-total-ao-disco-opcional) explica o resto.

O app funciona sem a permissão. O macOS só deixa essas pastas fora dos resultados.

## Como toda remoção funciona

Quatro telas podem remover coisas: "Limpeza", "Duplicados", "Fotos" e "Desenvolvedor". Todas seguem os mesmos passos, então quem aprende uma já sabe usar as outras.

1. **A tela procura primeiro.** Procurar e analisar nunca removem nada.
2. **Você vê o que foi encontrado** e escolhe o que sai. Alguns itens já vêm selecionados; você pode mudar todas as escolhas.
3. **Você clica no botão de ação,** embaixo, à direita. Um diálogo chamado "Autorização requerida" diz quantos itens, quanto espaço, e se eles vão para a Lixeira ou são removidos de vez.
4. **Você confirma, ou clica em "Cancelar".** Nada foi removido até você confirmar.
5. **"Removendo".** Cada item é conferido de novo logo antes de ser removido. Se mudou desde que você o viu, é pulado.
6. **"Concluído".** O diálogo diz quanto foi liberado, quantos itens foram removidos e gravados no histórico, e quantos foram deixados como estavam. Clique em "Fechar"; a tela então procura de novo, para a lista ficar atualizada.

Se você clicar no botão de ação sem nada selecionado, a tela mostra "Selecione ao menos um item." em vez de abrir o diálogo.

O [guia de segurança](safety.pt-BR.md) explica as regras por trás desses passos.

## As sete telas

### 1. "Painel"

O resumo da máquina e do que as outras telas encontraram. É a imagem do topo desta página.

| Parte | O que mostra |
|---|---|
| "Disco" | Quão cheio está o disco de inicialização e quanto há livre. |
| "Memória" | Quanta memória está em uso. |
| "Temperatura" | O sensor de temperatura mais quente. Quando o Mac não informa nenhum sensor, esse medidor mostra o uso do processador, com o título "CPU". |
| "Espaço recuperável" | A soma do que as outras telas encontraram, com uma linha por origem. |
| "Últimas remoções" | As entradas mais recentes do histórico de remoções: quando, se o item foi "apagado" ou está "na Lixeira", onde ele ficava e o tamanho. |
| "Sistema" | Um radar, algumas leituras e o botão "Analisar limpeza". |

As linhas de "Espaço recuperável":

| Linha | Vem da tela |
|---|---|
| "Caches e temporários" | "Limpeza" |
| "Projetos de código" | "Desenvolvedor" |
| "Arquivos duplicados" | "Duplicados" |
| "Fotos parecidas" | "Fotos" |

Uma linha mostra "não analisado" até a tela dela ter rodado. **Clique em uma linha para abrir a tela correspondente.**

O botão **"Analisar limpeza"** roda a análise da limpeza ali mesmo, sem sair do painel. Ele só mede; não remove nada. Quando termina, a linha "Caches e temporários" mostra o resultado.

### 2. "Limpeza"

Caches, arquivos temporários, logs e outras sobras que os apps e as ferramentas recriam quando precisam.

![A tela de limpeza: uma lista de regras com etiqueta de risco, barra de tamanho e o tamanho de cada uma](../assets/screens/cleanup.png)

Passo a passo:

1. Pressione `2`. Na primeira vez que a tela abre, a análise começa sozinha. Aparece "Analisando". **A análise não remove nada.**
2. Leia a lista. Cada linha é uma regra: uma caixa de seleção, o nome, onde ela procura, uma etiqueta e quanto espaço encontrou. Uma linha que não encontrou nada mostra "vazio" e não pode ser selecionada.
3. Olhe as etiquetas. "Seguro" quer dizer que o conteúdo é recriado automaticamente. "Revisar" quer dizer que pode valer a pena guardar, então olhe antes de selecionar.
4. Clique em uma linha para selecionar ou tirar a seleção. Embaixo, à esquerda, aparece o total: "7 selecionados · 108,7 MB".
5. Clique em **"Limpar selecionados"**.
6. Leia o diálogo "Autorização requerida". Depois clique em **"Autorizar limpeza"**, ou em "Cancelar".
7. Leia o resultado em "Concluído" e clique em "Fechar".

Uma regra remove o **conteúdo** das pastas dela, nunca as pastas em si.

#### O grupo "Conta do usuário"

"Conta do usuário" reúne o que a sua própria conta pode remover, sem autenticação. As regras seguras que encontraram alguma coisa já vêm selecionadas.

| Regra na tela | O que remove | Etiqueta |
|---|---|---|
| "Caches de aplicativos" | O conteúdo de `~/Library/Caches` | "Seguro" |
| "Arquivos temporários" | O conteúdo da sua pasta temporária, só itens que não mudam há 7 dias ou mais | "Seguro" |
| "Logs e relatórios de falha" | O conteúdo de `~/Library/Logs` | "Seguro" |
| "Lixeira" | Esvazia a Lixeira. O macOS só mostra a Lixeira a apps com Acesso Total ao Disco; sem ele, essa linha mostra "vazio". | "Revisar" |
| "Xcode DerivedData" | O conteúdo de `~/Library/Developer/Xcode/DerivedData` (resultado de compilações) | "Seguro" |
| "Caches do simulador" | O conteúdo de `~/Library/Developer/CoreSimulator/Caches` | "Seguro" |
| "Suporte a dispositivos do Xcode" | O conteúdo de `~/Library/Developer/Xcode/iOS DeviceSupport`. O Xcode baixa de novo quando você conecta o aparelho. | "Revisar" |
| "Xcode Archives" | O conteúdo de `~/Library/Developer/Xcode/Archives`. **Esses vão para a Lixeira**, não são removidos de vez. | "Revisar" |
| "Caches de gerenciadores de pacotes" | Os caches do npm, cargo, Gradle, NuGet e pnpm: `~/.npm/_cacache`, `~/.cargo/registry/cache`, `~/.cargo/registry/src`, `~/.gradle/caches`, `~/.nuget/packages`, `~/Library/pnpm/store` | "Seguro" |

> Cuidado com a "Lixeira": esvaziar a Lixeira não tem volta, e é para lá que o MacCleaner manda duplicados e fotos. Por isso essa regra nunca vem selecionada. Selecione só quando tiver certeza de que não quer recuperar nada de lá.

#### O grupo "Sistema"

"Sistema" reúne duas áreas que só um administrador pode alterar. **Elas nunca vêm selecionadas.**

| Regra na tela | O que remove | Etiqueta |
|---|---|---|
| "Caches do sistema" | O conteúdo de `/Library/Caches` | "Seguro" |
| "Logs do sistema" | O conteúdo de `/Library/Logs` | "Seguro" |

O que acontece quando você seleciona uma delas e confirma:

1. O diálogo "Autorização requerida" ganha a linha "As áreas de sistema pedem Touch ID ou a senha de administrador, no diálogo do próprio macOS".
2. Depois que você clica em "Autorizar limpeza", as regras da sua própria conta são limpas primeiro.
3. Em seguida, **o macOS mostra o diálogo de autenticação dele**, com o texto "O MacCleaner precisa de permissão de administrador para limpar as áreas de sistema que você selecionou.". Use o Touch ID ou digite a senha de administrador. A senha vai para o macOS; o MacCleaner nunca a vê.
4. Se você fechar esse diálogo, as áreas de sistema ficam intocadas e o resultado diz "As áreas de sistema não foram limpas: a autenticação foi cancelada.". O que foi limpo no passo 2 continua limpo.

O que saber antes de usar:

- **Precisa do app instalado.** A limpeza de sistema é feita por um pequeno programa auxiliar que o `cargo xtask install` coloca dentro do app. Quando o app roda direto do código sem esse auxiliar, o grupo mostra "só no app instalado · cargo xtask install" e as linhas dele não podem ser selecionadas.
- **Touch ID ou senha, quem decide é o macOS,** não o MacCleaner.
- **Não foi verificado de ponta a ponta.** O auxiliar é coberto por testes automáticos em pastas temporárias, mas o diálogo do macOS e uma limpeza real de `/Library/Caches` e `/Library/Logs` não foram exercitados pelos testes do projeto. Veja o [guia de segurança](safety.pt-BR.md#áreas-de-administrador).
- Itens que o próprio macOS protege (Proteção da Integridade do Sistema, SIP) não podem ser removidos nem por um administrador. Eles aparecem como deixados como estavam.

### 3. "Duplicados"

Arquivos com exatamente o mesmo conteúdo, guardados mais de uma vez.

![A tela de duplicados: grupos de arquivos idênticos, com a cópia a manter marcada com "Manter"](../assets/screens/duplicates.png)

Passo a passo:

1. Pressione `3`.
2. Escolha o "Tamanho mínimo": "1 MB", "10 MB" ou "100 MB". Arquivos menores são ignorados. Um mínimo maior deixa a busca mais rápida.
3. Clique em **"Escolher pasta"** e escolha uma pasta, ou clique em **"Pasta pessoal"** para procurar na sua pasta pessoal inteira.
4. Espere o "Procurando duplicados". "Cancelar" interrompe a busca.
5. Leia os grupos. Cada grupo é um arquivo que existe em vários lugares: o nome, quantas cópias e o tamanho de cada uma. Em cada grupo, uma cópia desmarcada mostra a etiqueta **"Manter"** e fica; as marcadas vão sair. Por padrão, a cópia mantida é a mais antiga.
6. Clique em uma linha para marcar ou desmarcar. Você pode manter outra cópia, ou manter todas.
7. Clique em **"Mover para a Lixeira"**, leia o diálogo e clique em "Mover para a Lixeira" de novo para confirmar.

O que a tela garante:

- **Ao menos uma cópia de cada arquivo sempre fica.** Se você tentar marcar a última, a tela recusa e mostra "Ao menos uma cópia de cada arquivo precisa ficar."
- **As cópias vão para a Lixeira.** Dá para recuperar de lá. O espaço em disco só é liberado quando você esvazia a Lixeira.
- **Dois arquivos só contam como duplicados quando o conteúdo inteiro é idêntico.** Os arquivos são comparados primeiro pelo tamanho, depois pelo começo e pelo fim, depois pelo conteúdo inteiro (com o hash BLAKE3). O nome não importa.
- **Hard links não contam como cópias.** Dois nomes para o mesmo arquivo gravado ocupam o espaço uma vez só, então remover um deles não liberaria nada.
- **Arquivos que estão só no iCloud nunca são lidos.** Compará-los exigiria baixá-los. Arquivos vazios, o interior de apps e de outros pacotes e os atalhos (links simbólicos) também ficam de fora.

São mostrados no máximo 100 grupos, primeiro os que liberam mais espaço. Quando há mais, a tela mostra "Mais N grupos menores não são mostrados. Remova os maiores e procure de novo."

Depois do resultado, "Procurar de novo" repete a busca na mesma pasta, e "Escolher pasta" começa em outra.

### 4. "Fotos"

Fotos que parecem a mesma, ainda que os arquivos sejam diferentes: uma cópia redimensionada, a mesma imagem salva em outro formato, fotos tiradas em sequência.

![A tela de fotos: grupos de fotos parecidas com miniaturas, a foto de referência e o percentual de correspondência](../assets/screens/photos.png)

Passo a passo:

1. Pressione `4`.
2. Escolha o nível de "Semelhança":

   | Nível | O que reúne |
   |---|---|
   | "Quase idênticas" | A mesma imagem salva de novo: outro tamanho, formato ou compressão. |
   | "Parecidas" | Também imagens com pequenas diferenças, como fotos tiradas em sequência. É o padrão. |
   | "Ampla" | Também imagens que só lembram uma à outra. Revise com cuidado. |

3. Clique em **"Escolher pasta"** e escolha uma pasta, ou clique em **"Procurar em Imagens"** para procurar na sua pasta Imagens.
4. Espere o "Procurando fotos parecidas". "Cancelar" interrompe a busca.
5. Leia os grupos. Cada grupo mostra as fotos lado a lado:
   - A primeira vem marcada com **"REFERÊNCIA"**. É o maior arquivo, provavelmente o original, e é mantida por padrão.
   - As outras mostram **"MATCH n%"**: o quanto se parecem com a referência.
   - Embaixo de cada imagem ficam o tamanho, a pasta e "Manter" ou "Remover".
6. **Clique em uma imagem para alternar entre "Manter" e "Remover".**
7. Clique em **"Mover para a Lixeira"**, leia o diálogo e confirme.

O que a tela garante:

- **Ao menos uma foto de cada grupo sempre fica.** Senão a tela mostra "Ao menos uma foto de cada grupo precisa ficar."
- **As fotos marcadas vão para a Lixeira.** Dá para recuperar de lá.
- **A biblioteca do app Fotos nunca é lida nem alterada.** O macOS guarda essa biblioteca como um pacote lacrado, e o MacCleaner não o abre. Só imagens soltas em pastas são comparadas.
- **Arquivos que estão só no iCloud nunca são abertos.**

O que é comparado:

- Imagens destes formatos: JPEG, PNG, HEIC/HEIF, TIFF, BMP, GIF, WebP e RAW de câmera (DNG, CR2, CR3, NEF, ARW, RAF, ORF, RW2, SRW, PEF).
- Só arquivos com pelo menos 50 KB. Imagens menores costumam ser ícones, não fotos.
- Um arquivo RAW ao lado de uma cópia revelada com o mesmo nome (`IMG_1.CR2` ao lado de `IMG_1.JPG`) é pulado: o par anda junto.

Como as fotos são comparadas: cada imagem é reduzida a uma pequena assinatura, feita do contorno, de um mapa do brilho, da cor média e das proporções. Duas fotos só combinam quando os quatro concordam. É por isso que cópias redimensionadas ou recomprimidas são encontradas, enquanto imagens cortadas, espelhadas e páginas de texto diferentes não são tratadas como a mesma.

São mostrados no máximo 60 grupos, primeiro os que liberam mais espaço. Uma imagem que não pode ser desenhada mostra "sem prévia".

### 5. "Desenvolvedor"

Para quem escreve código: as pastas de build e de dependências que as ferramentas recriam sozinhas, como `node_modules`, `target` e `bin`/`obj`. Se você não escreve código, pode ignorar esta tela.

![A tela do desenvolvedor: pastas de build e de dependências listadas por projeto, com o ecossistema e a etiqueta de risco](../assets/screens/developer.png)

Passo a passo:

1. Pressione `5`.
2. Clique em **"Escolher pasta"** e escolha a pasta onde ficam os seus projetos, ou clique em **"Pasta pessoal"** para procurar na sua pasta pessoal.
3. Espere o "Procurando projetos".
4. Leia a lista "Pastas recriáveis". Cada linha é `projeto › pasta`, com o ecossistema, quando o projeto foi alterado pela última vez ("alterado há 2 meses"), uma etiqueta e o tamanho. Os botões acima da lista ("Todos", "Node", "Rust"...) filtram por ecossistema.
5. Confira o que está selecionado. **Pastas com a etiqueta "Seguro" em projetos parados há 30 dias ou mais já vêm selecionadas.** Todo o resto espera por você.
6. Clique em **"Excluir selecionados"**, leia o diálogo e clique em **"Autorizar exclusão"**.

**Essa remoção é permanente.** As pastas não vão para a Lixeira. Elas voltam na próxima vez que você instalar as dependências ou compilar o projeto. Nenhum código-fonte é removido.

Uma pasta só é oferecida quando as duas coisas são verdade:

- **O arquivo que identifica o projeto está bem ao lado dela.** Uma pasta chamada `target` só é oferecida ao lado de um `Cargo.toml` ou de um `pom.xml`, por exemplo.
- **O git não tem nenhum arquivo versionado dentro dela.** Se parte da pasta está sob controle de versão, ela é conteúdo do projeto, não saída de build. Quando o git não consegue responder, a pasta não é oferecida.

O que é reconhecido:

| Ecossistema | Reconhecido por | Pastas "Seguro" | Pastas "Revisar" |
|---|---|---|---|
| Node | `package.json` | `node_modules`, `.next`, `.nuxt`, `.turbo`, `.angular`, `.parcel-cache`, `.svelte-kit` | `dist`, `build`, `coverage`, `out` |
| .NET | `*.csproj`, `*.fsproj`, `*.vbproj`, `*.sln` | `bin`, `obj`, `.vs`, `TestResults` | `packages` |
| Rust | `Cargo.toml` | `target` | |
| Java/Kotlin (Maven) | `pom.xml` | `target` | |
| Java/Kotlin (Gradle) | `build.gradle`, `build.gradle.kts`, `settings.gradle`, `settings.gradle.kts` | `build`, `.gradle` | |
| Python | `pyproject.toml`, `requirements.txt`, `setup.py`, `Pipfile` | `.venv`, `venv`, `__pycache__`, `.pytest_cache`, `.mypy_cache`, `.ruff_cache`, `.tox` | `build`, `dist` |
| Swift (pacotes) | `Package.swift` | `.build` | |
| Swift (CocoaPods) | `Podfile` | `Pods` | |
| Flutter | `pubspec.yaml` | `.dart_tool` | `build` |
| Go | `go.mod` | | `vendor` |
| PHP | `composer.json` | | `vendor` |
| Terraform | `*.tf` | `.terraform` | |

As pastas "Revisar" têm nomes comuns, que um projeto pode usar para conteúdo de verdade, então nunca vêm selecionadas.

Quando você procura na pasta pessoal, as pastas Biblioteca, Aplicativos, Imagens, Filmes, Música e a Lixeira são puladas: não guardam projetos de código e são lentas de ler.

### 6. "Mapa do disco"

Um mapa do que ocupa o disco. Cada bloco tem o tamanho do espaço que ocupa. O app chama a tela de "Mapa de setores". Esta tela só lê: nada pode ser apagado por ela.

![O mapa do disco: blocos com o tamanho do espaço que cada pasta ocupa e, embaixo, a legenda dos tipos de arquivo](../assets/screens/disk-map.png)

Passo a passo:

1. Pressione `6`.
2. Clique em **"Pasta pessoal"** para medir a sua pasta pessoal, ou em **"Disco inteiro"** para medir o disco de inicialização. O macOS pode pedir permissão para ler Documentos, Mesa e Downloads.
3. Espere o "Varrendo". A nota "iCloud: sem downloads" lembra que nada está sendo baixado. "Cancelar" interrompe a varredura.
4. **Passe o cursor sobre um bloco** para ver a ficha dele: "Tamanho" (o tamanho e a fatia do mapa), "Itens", "iCloud" (onde o conteúdo está) e "Tipo".
5. **Clique em um bloco de pasta para ampliar e entrar nela.**
6. **Use a trilha de nomes acima do mapa para voltar.** Clique em qualquer nome dela para retornar àquela pasta.
7. **Clique com o botão direito em um bloco para mostrá-lo no Finder.**

Os controles acima do mapa:

| Controle | O que faz |
|---|---|
| "Cor por" "Tipo" | Pinta cada bloco pelo tipo de conteúdo: "Sistema e apps", "Código", "Fotos", "Vídeo e áudio", "Documentos", "Compactados", "Outros". |
| "Cor por" "iCloud" | Pinta cada bloco por onde ele está: "Somente local" (só neste Mac), "iCloud, baixado" (no iCloud e também neste Mac), "Somente na nuvem" (só no iCloud, sem ocupar espaço aqui). |
| "Varrer novamente" | Mede o mesmo lugar de novo. |

A linha abaixo do mapa mostra quantos itens foram medidos, o tamanho total e quanto a varredura demorou. Se o macOS não deixou o app abrir algumas pastas, ela ganha "N pastas sem acesso" e o link "Abrir ajustes de Acesso Total ao Disco".

Vale saber:

- **A varredura nunca inicia um download do iCloud.** Arquivos que estão só no iCloud são medidos e identificados, não baixados.
- Cada nível mostra no máximo 40 blocos. Os menores são reunidos em um bloco chamado "outros".
- Apps, bibliotecas do Fotos e outros pacotes aparecem como um bloco lacrado.
- "Disco inteiro" deixa duas coisas de fora de propósito: outros discos conectados ao Mac e o volume interno que o macOS mostra duas vezes, o que contaria cada arquivo em dobro.

### 7. "Monitor"

A máquina, ao vivo. O app chama esta tela de "Telemetria".

![O monitor: traço do processador, uma barra por núcleo, memória, disco, temperaturas e os programas mais ativos](../assets/screens/monitor.png)

| Quadro | O que mostra |
|---|---|
| "CPU" | O uso do processador nos últimos dois minutos e o percentual atual. |
| "Núcleos" | Uma barra por núcleo do processador. |
| "Memória" | Memória em uso em relação ao total, "Livre" e "Swap em uso". |
| "Disco" | Espaço usado no disco de inicialização em relação ao total, e "Livre". |
| "Temperatura" | Temperaturas agrupadas por componente: "CPU", "GPU", "SSD", "Bateria", "Placa". Só aparecem os componentes que o seu Mac informa. |
| "Processos" | Os seis programas que mais usam o processador, com a memória de cada um. Aqui 100% é um núcleo inteiro, então um programa pode passar de 100%. |

De quanto em quanto tempo as leituras mudam: processador e memória a cada segundo, a lista de programas a cada 2 segundos, temperaturas a cada 5 segundos.

Vale saber:

- Se o Mac não informa nenhum sensor de temperatura, o quadro mostra "Este Mac não informou nenhum sensor de temperatura."
- Não há velocidade das ventoinhas.
- Esta tela só lê. Ela não encerra programas.

## A ferramenta de linha de comando

O `maccleaner-cli` roda o mesmo motor de varredura do app, pelo Terminal. Ele é somente leitura: nunca altera nem apaga nada.

Rode a partir da pasta do projeto:

```bash
cargo run --release -p maccleaner-cli -- scan ~/Documents
```

### O que o `scan` aceita

```text
maccleaner-cli scan [--top <N>] <PASTA>
```

| Parte | Significado | Padrão |
|---|---|---|
| `<PASTA>` | A pasta a medir | obrigatório |
| `--top <N>` | Quantos dos maiores itens listar | 15 |

Exemplos:

```bash
cargo run --release -p maccleaner-cli -- scan ~
```

```bash
cargo run --release -p maccleaner-cli -- scan ~/Projetos --top 30
```

### Como ler o resultado

```text
Pasta              /Users/voce/Projetos
Itens              1.386.543
Ocupado em disco   24,2 GB
Tamanho lógico     20,6 GB
Somente na nuvem   0 B
Pastas sem acesso  0
Tempo              9,3 s

Maiores itens
     12,5 GB  portal-web/
     11,7 GB  app-mobile/
    249,9 KB  notas.txt
```

| Linha | Significado |
|---|---|
| Pasta | A pasta que foi medida. |
| Itens | Quantos arquivos e pastas há dentro dela. |
| Ocupado em disco | O espaço que esses itens realmente tomam no disco. É o número que importa quando você quer liberar espaço. |
| Tamanho lógico | O tamanho do conteúdo em si. Difere da linha de cima porque o disco guarda arquivos em blocos e pode compactá-los. |
| Somente na nuvem | Conteúdo que existe só no iCloud. Não ocupa espaço neste Mac, e o MacCleaner não o baixa. |
| Pastas sem acesso | Pastas que o macOS não deixou o Terminal abrir. Veja a próxima seção. |
| Tempo | Quanto a varredura demorou. |
| Maiores itens | Os maiores itens diretamente dentro da pasta. Uma `/` depois do nome indica pasta; "(pacote)" indica um app ou uma biblioteca medidos como um item só. |

### Quando "Pastas sem acesso" não é zero

O macOS protege algumas pastas, como as do Mail, do Safari e os dados de outros apps. A ferramenta de linha de comando roda dentro do Terminal, então aqui é o Terminal que precisa do **Acesso Total ao Disco**, não o MacCleaner:

1. Abra os **Ajustes do Sistema**.
2. Vá em **Privacidade e Segurança** e depois em **Acesso Total ao Disco**.
3. Ligue a chave ao lado de **Terminal** (ou do app de terminal que você usa).
4. Encerre o Terminal e abra de novo.

Isso só permite que o Terminal leia essas pastas. Você pode desligar do mesmo jeito.

### Medindo o disco inteiro

```bash
cargo run --release -p maccleaner-cli -- scan /
```

Demora mais e precisa do Acesso Total ao Disco para ficar completo. Duas coisas ficam de fora de propósito: outros discos conectados ao Mac e o volume interno que o macOS mostra duas vezes, o que contaria cada arquivo em dobro.

## A seguir

- [Segurança](safety.pt-BR.md): o que o MacCleaner pode e não pode apagar, e o que ainda não foi testado.
- [Perguntas e solução de problemas](faq.pt-BR.md).
